const fs = require('node:fs')
const net = require('node:net')
const path = require('node:path')
const { randomUUID } = require('node:crypto')

// shortcut: one loopback lock serializes all local fixture writes; use per-catalog locking if parallel fixtures need it.
const revokeLockPort = 18084

const [action, ...args] = process.argv.slice(2)
const expected = action === 'serve' ? ['--port', '--catalog'] : ['--catalog', '--destination']
const options = new Map()
for (let i = 0; i < args.length; i += 2) {
  if (!expected.includes(args[i]) || options.has(args[i]) || !args[i + 1]) {
    throw new Error('Invalid fixture options.')
  }
  options.set(args[i], args[i + 1])
}
const catalog = options.get('--catalog')
if (!['serve', 'revoke'].includes(action) || options.size !== expected.length || !path.isAbsolute(catalog)) {
  throw new Error('Use serve --port PORT --catalog PATH or revoke --catalog PATH --destination ID.')
}

function publish(snapshot, initial = false) {
  const temporary = catalog + '.' + randomUUID() + '.tmp'
  fs.writeFileSync(temporary, JSON.stringify(snapshot), { flag: 'wx', mode: 0o600 })
  try {
    if (initial) fs.linkSync(temporary, catalog)
    else fs.renameSync(temporary, catalog)
  } finally {
    fs.rmSync(temporary, { force: true })
  }
}

async function main() {
  if (action === 'revoke') {
    // The OS releases the listener even if this process is forcibly terminated.
    const lock = net.createServer(socket => socket.destroy())
    try {
      await new Promise((resolve, reject) => {
        lock.once('error', reject)
        lock.listen(revokeLockPort, '127.0.0.1', resolve)
      })
    } catch (error) {
      if (error.code === 'EADDRINUSE') throw new Error('Catalog update locked. Retry when loopback port 18084 is free.')
      throw error
    }
    try {
      const snapshot = JSON.parse(fs.readFileSync(catalog, 'utf8'))
      const id = options.get('--destination')
      const destinations = snapshot.destinations.filter(entry => entry.id !== id)
      if (destinations.length === snapshot.destinations.length) throw new Error('Destination is not active.')
      const version = snapshot.version + 1
      publish({ version, destinations })
      console.log('Published catalog version ' + version)
    } finally {
      await new Promise(resolve => lock.close(resolve))
    }
    return
  }

  const firstPort = Number(options.get('--port'))
  if (!Number.isInteger(firstPort) || firstPort < 1 || firstPort > 65533) throw new Error('Invalid start port.')
  if (firstPort <= revokeLockPort && revokeLockPort <= firstPort + 2) throw new Error('Loopback port 18084 is reserved for catalog updates.')
  const scenarios = [
    ['fixture-ok', 'Correct identity', 'fixture-agent', 0],
    ['fixture-slow', 'Delayed response', 'fixture-agent', 11000],
    ['fixture-wrong', 'Different identity', 'different-agent', 0],
  ]
  const servers = []
  const destinations = []
  try {
    for (const [offset, [id, label, identity, delay]] of scenarios.entries()) {
      let contacts = 0
      const port = firstPort + offset
      const server = net.createServer(socket => {
        socket.on('error', () => socket.destroy())
        console.log(id + ' contacts=' + ++contacts)
        const send = setTimeout(() => socket.end(JSON.stringify(identity)), delay)
        socket.on('close', () => clearTimeout(send))
      })
      servers.push(server)
      await new Promise((resolve, reject) => {
        server.once('error', reject)
        server.listen(port, '127.0.0.1', resolve)
      })
      destinations.push({ id, label, port, version: randomUUID() })
    }
    publish({ version: 1, destinations }, true)
    console.log('Local fixture ready; published catalog version 1.')
  } catch (error) {
    for (const server of servers) server.close()
    throw error
  }
}
main().catch(error => {
  console.error(error.message)
  process.exitCode = 1
})
