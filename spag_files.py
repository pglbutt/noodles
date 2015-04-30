import os
import yaml

from common import ToughNoodles, split_path


def load_file(filename):
    if not os.path.exists(filename):
        raise ToughNoodles('File {0} not found'.format(filename))
    with open(filename, 'r') as f:
        return yaml.load(f)


class SpagFilesLookup(dict):
    """
    A dict for looking up our request files by the file's unqualified name.
    e.g. if we have directory tree that looks like:

        myrequests/
            v1/
                create_thing.yml
                delete_thing.yml
            v2/
                create_thing.yml
                delete_thing.yml

    Then this dictionary looks like:

        {'create_thing.yml': ['/user/dir/myrequests/v1/create_thing.yml',
                              '/user/dir/myrequests/v2/create_thing.yml'],
         'delete_thing.yml': ['/user/dir/myrequests/v1/delete_thing.yml',
                              '/user/dir/myrequests/v2/delete_thing.yml]}
    """

    INCLUDED_EXTENSIONS = ['.yml', '.yaml']

    @classmethod
    def has_valid_extension(cls, name):
        return any(name.endswith(e) for e in cls.INCLUDED_EXTENSIONS)

    VALID_EXTENSIONS = ['.yml', '.yaml']

    @classmethod
    def has_valid_extension(cls, name):
        return any(name.endswith(e) for e in cls.VALID_EXTENSIONS)

    def __init__(self, *dirs):
        self.dirs = set([])
        for dir in dirs:
            self.add_dir(dir)

    def add_dir(self, dir):
        # convert to absolute paths and use sets to handle duplicates
        absdir = os.path.abspath(dir)
        if not os.path.exists(absdir):
            raise ToughNoodles("Directory %s not found")

        self.dirs.add(absdir)

        for dirname, _, files in os.walk(dir):
            absdirname = os.path.abspath(dirname)
            for filename in files:
                # ignore unsupported extensions
                if not self.has_valid_extension(filename):
                    continue
                fullpath = os.path.join(absdirname, filename)
                if filename not in self:
                    self[filename] = set([fullpath])
                else:
                    self[filename].add(fullpath)

    def get_path(self, key):
        """Get the unique path stored under the given key.

        If self is:
            {'req.yml': set(['/a/b/v1/req.yml',
                             '/a/b/v2/req.yml'])}

        Then:
            get_path('v1/req.yml') -> '/a/b/v1/req.yml'
            get_path('v2/req.yml') -> '/a/b/v2/req.yml'
            get_path('req.yml') -> raises ToughNoodles
        """
        key = key.strip('/')

        # 'a/b/c.yml' -> ('a', 'b', 'c.yml')
        key_parts = split_path(key)

        # lookup the unqualified filename
        paths = self.get(key_parts[-1])
        if not paths:
            raise ToughNoodles("No files matching '{0}'".format(key_parts[-1]))
        if len(paths) == 1:
            return next(iter(paths))

        # if key = 'a/b/c.yml', look for the path ending with 'a/b/c.yml'
        matches = [path for path in paths
                   if split_path(path)[-len(key_parts):] == key_parts]

        if len(matches) > 1:
            raise ToughNoodles("Ambiguous request name. Pick from {0}"
                               .format(matches))
        elif not matches:
            raise ToughNoodles("Invalid request name {0}".format(key))
        else:
            return matches[0]
