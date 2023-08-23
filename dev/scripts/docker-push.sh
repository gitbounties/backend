#!/bin/sh

docker build -t pinosaur/gitbounties_backend:latest -f Dockerfile.local .
docker push pinosaur/gitbounties_backend:latest
