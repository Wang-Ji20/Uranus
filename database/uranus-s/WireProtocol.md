# Wire Protocol

Uranus uses a simple wire protocol resemble to Redis's [RESP](https://redis.io/docs/reference/protocol-spec/) protocol.

The aim is the same as Redis's:
- easy to parse and read
- simple to implement

## Network

Clients and servers are connected by TCP. The server only responses to a client when it had issued a request. There's no server side events or pushes in this protocol.

## Specification

I use "\r\n" as the delimiter. Every part of the request/response is divided by it.

\+: Simple string type
    A "\r\n" terminates the string.

\-: Error type
    Only server send this to client.

\*: String array type
    After \* is the size of the array, then "\r\n".
    Each string is delimited by "\r\n".

$: Binary type
    After $ is the size of binary data.


