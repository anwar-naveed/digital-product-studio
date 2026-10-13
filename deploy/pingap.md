# deploy/pingap.md — Pingap route + certificate recipe for studio.amzankos.local

Target: **webtop-internal only** (same pattern as `openship.amzankos.local`).
No public DNS, no Let's Encrypt, no public port. Accessed only from the webtop
browser where the Amzanko CA is trusted.

## 1. Certificate

Use the existing Amzanko Local Root CA:
`/config/.hermes/workspace/projects/amzankos-trader/backend/devops/ca/myCA.crt` (+ `.key`, `.pem`).

```bash
CA=/config/.hermes/workspace/projects/amzankos-trader/backend/devops/ca
CERTS=/config/.hermes/workspace/projects/amzankos-trader/pingap/certs

# leaf key + CSR
openssl req -new -newkey rsa:2048 -nodes \
  -keyout "$CERTS/studio.amzankos.local.key" \
  -subj "/CN=studio.amzankos.local" \
  -addext "subjectAltName=DNS:studio.amzankos.local" \
  -out /tmp/studio.csr

# sign with the CA (SAN carried from CSR)
openssl x509 -req -in /tmp/studio.csr \
  -CA "$CA/myCA.crt" -CAkey "$CA/myCA.key" -CAcreateserial \
  -days 825 -sha256 \
  -copy_extensions copyall \
  -out "$CERTS/studio.amzankos.local.crt"

# fullchain = leaf + CA (same layout as the other certs)
cat "$CERTS/studio.amzankos.local.crt" "$CA/myCA.crt" \
  > "$CERTS/studio.amzankos.local.fullchain.crt"
rm /tmp/studio.csr
```

Verify: `openssl verify -CAfile "$CA/myCA.crt" "$CERTS/studio.amzankos.local.fullchain.crt"`.

(Alternative per doc 08 §7: self-signed leaf exactly like the current
`openship.amzankos.local` cert — then the browser needs a one-time exception.
The myCA path avoids that.)

## 2. Pingap config additions

Files live in `/config/.hermes/workspace/projects/amzankos-trader/pingap/conf/`
(bind-mounted into the `amzankos-pingap` container). Host paths for any file
ops: prefix with `/opt/vps-ui/webtop/config/`.

### certificates.toml
```toml
[certificates.studio]
domains = "studio.amzankos.local"
tls_cert = "/etc/pingap/certs/studio.amzankos.local.fullchain.crt"
tls_key = "/etc/pingap/certs/studio.amzankos.local.key"
```

### upstreams.toml
```toml
[upstreams.studioUpstream]
addrs = ["172.19.0.1:3081"]          # admin-ui (nginx) published on the gateway
health_check = "tcp://172.19.0.1:3081"
health_check_interval = "10s"
health_check_timeout = "5s"

[upstreams.studioApiUpstream]
addrs = ["172.19.0.1:3082"]          # pipeline-core API on the gateway
health_check = "http://172.19.0.1:3082/api/v1/health"
health_check_interval = "10s"
health_check_timeout = "5s"
```

### locations.toml
```toml
[locations.studioUi]
host = "studio.amzankos.local"
path = "/"
plugins = ["requestId"]
proxy_set_headers = [
    "Host:$host",
    "X-Real-IP:$remote_addr",
    "X-Forwarded-For:$proxy_add_x_forwarded_for",
    "X-Forwarded-Proto:$scheme",
    "X-Forwarded-Host:$host",
    "X-Forwarded-Port:$server_port",
]
upstream = "studioUpstream"

[locations.studioApi]
host = "studio.amzankos.local"
path = "/api/"
plugins = ["requestId"]
proxy_set_headers = [
    "Host:$host",
    "X-Real-IP:$remote_addr",
    "X-Forwarded-For:$proxy_add_x_forwarded_for",
    "X-Forwarded-Proto:$scheme",
    "X-Forwarded-Host:$host",
]
upstream = "studioApiUpstream"
```

## 3. Apply

Pingap hot-reloads config files; if not, `docker restart amzankos-pingap`.

## 4. Verify (inside the webtop)

```bash
curl -sk -o /dev/null -w '%{http_code}\n' https://studio.amzankos.local/
curl -sk https://studio.amzankos.local/api/v1/health
```

Port registry addition: 3081 = admin-ui, 3082 = pipeline-core (on 172.19.0.1;
no conflict with 3080/3081 already in use — check `docker port` before wiring
and adjust the compose port map if 3081 is taken).
