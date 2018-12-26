# TODO

- SSL with a self-signed cert doesn't work, even though the irc crate seems to
  pass the cert to native-ssl for verification. Needs investigation.
- Must fetch URL over Tor. Either the SOCKS interface, or over Tor's new HTTP
  Proxy (HttpTunnelPort). But the latter *only* supports CONNECT.
  - Currently using privoxy to put a proper http proxy in front ov the Tor
    socks5 proxy.
