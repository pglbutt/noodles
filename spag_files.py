import os
import yaml
from common import ToughNoodles, split_path, update, ensure_dir_exists


def load_file(filename):
    if not os.path.exists(filename):
        raise ToughNoodles('File {0} not found'.format(filename))
    with open(filename, 'r') as f:
        return yaml.safe_load(f)


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

    VALID_EXTENSION = '.yml'

    @classmethod
    def has_valid_extension(cls, name):
        return name.endswith(cls.VALID_EXTENSION)

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

        # support looking up both 'do_thing' and 'do_thing.yml'
        if not self.has_valid_extension(key):
            key += self.VALID_EXTENSION

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

    def get_file_list(self):
        return list(sorted(os.path.relpath(path, '.')
                           for paths in self.values()
                           for path in paths))

class SpagEnvironment(object):

    SPAG_ENV_DIR = './.spag/environments/'
    DEFAULT_ENV_NAME = 'default.yml'

    @classmethod
    def activate(cls, envname):
        ensure_dir_exists(cls.SPAG_ENV_DIR)

        filename = os.path.join(cls.SPAG_ENV_DIR, envname + '.yml')
        try:
            env = load_file(filename)
        except Exception as e:
            raise ToughNoodles(e.message)

        active = os.path.join(cls.SPAG_ENV_DIR, 'active')
        f = open(active, 'w')
        f.write(filename)

    @classmethod
    def deactivate(cls):
        try:
            os.remove(cls.SPAG_ENV_DIR + 'active')
        except OSError:
            pass

    @classmethod
    def get_env(cls, envname=None):
        # Viewing an inactive environment
        if envname is not None:
            filename = os.path.join(cls.SPAG_ENV_DIR, envname + '.yml')
            try:
                return load_file(filename)
            except Exception as e:
                raise ToughNoodles(e.msg)

        # If there is no environment, activate the default one
        activename = os.path.join(cls.SPAG_ENV_DIR, 'active')
        if not os.path.exists(activename):
            cls._activate_default_env()

        # Load up and return the active environment
        f = open(activename, 'r')
        envname = os.path.join(f.read())
        return load_file(envname)


    @classmethod
    def set_env(cls, updates):
        activename = os.path.join(cls.SPAG_ENV_DIR, 'active')
        # If there is no active environment, use the default one
        if not os.path.exists(activename):
            cls._activate_default_env()

        f = open(activename, 'r')
        envname = f.read()
        current = load_file(envname)

        # Do a nested dictionary update, write and return the env
        current = update(current, updates)
        f = open(envname, 'w+')
        yaml.safe_dump(current, f, default_flow_style=False)
        return current

    @classmethod
    def unset_env(cls, var, everything=False):
        activename = os.path.join(cls.SPAG_ENV_DIR, 'active')
        if not os.path.exists(activename):
            cls._activate_default_env()

        f = open(activename, 'r')
        envname = f.read()
        current = load_file(envname)

        if everything is True:
            # If we're unsetting everything, just start over
            f = open(envname, 'w+')
            yaml.safe_dump({}, f, default_flow_style=False)
            return {}
        else:
            if var is None:
                return current

            # Attempt to find the variable and del it
            unset = cls._search_and_delete(current, var)
            if unset is None:
                raise ToughNoodles('Could not find anything in environment by that name')

            f = open(envname, 'w+')
            yaml.safe_dump(unset, f, default_flow_style=False)

            return unset

    @classmethod
    def _search_and_delete(cls, env, name):
        if name in env:
            del env[name]

        if 'headers' in env:
            if name in env['headers']:
                del env['headers'][name]

        if 'envvars' in env:
            if name in env['envvars']:
                del env['envvars'][name]

        return env

    @classmethod
    def _activate_default_env(cls):
        ensure_dir_exists(cls.SPAG_ENV_DIR)
        activename = os.path.join(cls.SPAG_ENV_DIR, 'active')

        if not os.path.exists(activename):
            envfile = os.path.join(cls.SPAG_ENV_DIR, cls.DEFAULT_ENV_NAME)
            if not os.path.exists(envfile):
                # Init the default env
                f = open(envfile, 'w+')
                yaml.safe_dump({}, f, default_flow_style=False)
                f.close()
            # Activate
            f = open(activename, 'w+')
            f.write(envfile)
            f.close()
