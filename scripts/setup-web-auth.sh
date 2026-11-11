#!/bin/sh
set -eu
ssh finn@192.168.5.5 python3 - <<'PY'
from pathlib import Path
import os,secrets
root=Path('/home/finn/.config/thetta')
root.mkdir(parents=True,exist_ok=True)
root.chmod(0o700)
for name in ['token_key','onshape_client_id','onshape_client_secret']:
    path=root/name
    if not path.exists():
        fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        with os.fdopen(fd,'wb') as file:
            if name=='token_key':file.write(secrets.token_bytes(32))
    path.chmod(0o600)
print('Thetta secret files are ready. Existing keys and credentials were preserved.')
PY
