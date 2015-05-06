import os
import errno
import collections

class ToughNoodles(Exception):
    pass

def split_path(path):
    path = path.strip('/')
    parts = []
    while True:
        path, last = os.path.split(path)
        if last:
            parts.append(last)
        else:
            if path:
                parts.append(path)
            break
    parts.reverse()
    return tuple(parts)

def ensure_dir_exists(dir):
    try:
        os.makedirs(dir)
    except OSError as e:
        # if dir already exists, we're good to go
        if e.errno == errno.EEXIST and os.path.isdir(dir):
            return
        raise common.ToughNoodles(str(e))

def update(d, u):
    if d is None:
        return u
    for k, v in u.items():
        if isinstance(v, collections.Mapping):
            r = update(d.get(k, {}), v)
            d[k] = r
        else:
            d[k] = u[k]
    return d