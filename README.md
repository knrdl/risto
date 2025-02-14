# risto
minimal shopping list webapp

vanilla / no frameworks

## test

```shell
docker run -it --rm -p8080:8080 ghcr.io/knrdl/risto
```

## deployment

Docker Compose:

```yaml
services:
  risto:
    image: ghcr.io/knrdl/risto
    hostname: risto
    restart: always
    ports:
      - 8080:8080
    volumes:
      - ./data:/data  # see data folder for examples
    cpus: 1s
    mem_limit: 250mb
```