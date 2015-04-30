import os

class ToughNoodles(Exception):
    pass

def split_path(path):
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
