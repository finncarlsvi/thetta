#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
remote=finn@192.168.5.5
./scripts/setup-web-auth.sh
ssh "$remote" 'mkdir -p /home/finn/thetta; if docker image inspect thetta:latest >/dev/null 2>&1; then docker tag thetta:latest thetta:previous; fi'
rsync -az --exclude .git --exclude target --exclude node_modules --exclude .claude --exclude .thetta-data --exclude research --exclude research-cam --exclude sampleDOTs ./ "$remote:/home/finn/thetta/"
ssh "$remote" 'cd /home/finn/thetta && docker compose build && docker compose up -d --wait'
curl --fail --silent --show-error http://192.168.5.5:3007/api/health
