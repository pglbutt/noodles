"""
Implements a simple template for parameterizing our requests.

Basic rules:

    - Use double braces to define a list: {{item1, item2, ...}}
    - Attempt to produce a value by evaluating each item in the list, in order.
    - The entire text "{{...}}" is replaced with the first item that
    successfully yields a value.
    - Each item in the list is evaluated as follows:
        1. Grabbing data from environments
            - "{{[env].headers.accept}}" says to lookup "headers.accept" in the
            environment "env.yml"
            - "{{[].headers.accept}}" says to lookup "headers.accept" in the
            active environment
        2. Grabbing data from requests
            - "{{post.response.body.id}}" says to lookup "response.body.id" in
            the request "post.yml"
            - We know to grab data from a request file if no env is specified
            and there is a '.' in the item
        3. Grabbing data from `--with` options
            - "{{thing_id}}" and "--with thing_id=poo" evaluates to "poo"
            - We know to look in the with parameters if there is no environment
            specified and there is no '.' in the item

Default values:

    - A default value can be specified using a colon:
        {{item1, item2 : default}}
    - This must be at the end of the list
    - If no items are matched, then the text after the colon is used
    - Whitespace surrounding the colon is ignored

Shortcut rules:

    - Double exclamation point:
        - If <expr> starts with "[env]" then @<expr> expands to {{[env]...}
            @[env].headers.accept --> {{[env].headers.accept}}
        - If <expr> does not contain a '.', then @<expr> expands to
        {{last.response.body.<expr>}}.
            /thing/@id --> /thing/{{last.response.body.id}}
        - If <expr> contains a '.', then "@<expr>" expands to
        {{last.response.<expr>}}.
            /thing/@headers.id --> /thing/{{last.response.headers.id}}
"""
import json
import string

from spag import common
from spag import remembers
from spag import files


def _find_double_braces(s):
    """Return (a, b) such that s[a:b] returns the first "{{...}}" in s.
    Return None on failure. Raise exception on unclosed double braces.
    """
    i = s.find("{{")
    if i < 0:
        return None
    j = s.find("}}", i)
    if j < 0:
        raise common.ToughNoodles("Unclosed braces: '{0}'".format(s[i:i+20]))
    return i, j + 2

def _read_environment_name(s):
    """Return (name, end) such that s[:end] == "[<name>]". name may be empty.

    Raises ToughNoodles on a dangling bracket, or if no bracket is found.
    """
    if not s:
        raise Exception("BUG: _read_environment_name should never receive an "
                        "empty string")
    if s[0] != '[':
        raise common.ToughNoodles("No brackets found in '{0}'".format(s))
    end = s.find(']')
    if end < 0:
        raise common.ToughNoodles("Unclosed bracket: '{0}'".format(s))
    return s[1:end], end + 1

def _substitute_braces(s, withs, body_type):
    """Replace a string "{{...}}" with a value. Raises ToughNoodles on bad
    formatting or failing to substitute.

    :param s: A string with a double-brace list: "{{...}}"
    :param withs: A dictionary containing the --with options
    :param body_type: If 'json', treat response.body as json
    """
    original = s
    assert s[:2] == "{{" and s[-2:] == "}}"
    s = s[2:-2]

    # don't allow any braces within double-brace sections
    # note: {{ is the escape sequence for a single { in a python string
    if '{' in s or '}' in s:
        raise common.ToughNoodles("'{{' and '}}' not allowed in double-braced "
                                  "list: '{0}'".format(original))
    if '@' in s:
        raise common.ToughNoodles("Shortcut '@' not allowed in double-braced "
                                  "list: '{0}'".format(original))

    items = [x.strip() for x in s.split(',') if x.strip()]

    # look for a default value
    default = None
    if items and ':' in items[-1]:
        # if the last item has multiple colons, split on the first one
        # e.g. 'item:my:weird:default'
        #      --> 'item' is the last item
        #      --> 'my:weird:default' is the last default
        parts = items[-1].split(':', 1)
        items[-1] = parts[0].strip()
        default = parts[1].strip()

        if not default:
            raise common.ToughNoodles(
                "Bad template - expected default value after ':' in {0}"
                .format(original))

    for item in items:
        if item.startswith('['):
            try:
                return _lookup_item_from_environment(item)
            except common.ToughNoodles:
                pass
        elif '.' in item:
            try:
                return _lookup_item_from_request(item, body_type)
            except common.ToughNoodles:
                pass
        elif item in withs:
            return withs[item]

    # haven't found a value yet. try returning the default.
    if default is not None:
        return default

    raise common.ToughNoodles("Failed to substitute for {0}".format(original))

def _dict_path_lookup(data, path):
    """If path == "a.b.c", return data['a']['b']['c']. Raise ToughNoodles if
    the value is not found.
    """
    path = path.strip('.').split('.')
    try:
        for key in path:
            # TODO: do we care about this?
            # this won't work if we have a list somewhere along the way:
            #   data = {a: [{b: 1}, {b: 2}]}
            #   path = "a.0.2"
            # we'll do:
            #   data = data['a']  # data == [{b: 1}, {b: 2}]
            #   data = data['0']  # error, list needs integer indices
            if key in data:
                data = data[key]
            else:
                raise common.ToughNoodles
        return data
    except TypeError:
        # raised if we tried LIST['poo'] or STRING['poo']
        pass
    except IndexError:
        # raised if we tried LIST[999999999] or STRING[99999999]
        pass
    raise common.ToughNoodles

def _lookup_item_from_environment(item):
    name, end = _read_environment_name(item)
    path = item[end:]
    if not path:
        raise common.ToughNoodles("No path found after [{0}]".format(item))

    name = name.strip() or None

    data = files.SpagEnvironment.get_env(name)
    # all of the following produce the same thing with _dict_path_lookup...
    #   "[env]poo"
    #   "[env].poo"
    #   "[env]...poo..."
    return _dict_path_lookup(data, path)

def _lookup_item_from_request(item, body_type):
    # for "thing.something.maybe.more", find thing.yml in the remembered files
    parts = item.split('.', 1)
    name, lookup_path = parts[0], parts[1]

    path = remembers.SpagRemembers().get_path(name)
    data = files.load_file(path)

    # load the body as json
    if body_type == 'json':
        try:
            body = data['response']['body']
            if 'body' in data['response'] and type(body) is str:
                data['response']['body'] = json.loads(body)
        except ValueError as e:
            pass
            # print("WARNING: {0} on item {1} with file {2}".format(str(e), item, path))
            # failed to load json, but keep trying. maybe the user
            # doesn't need the json body.

    return _dict_path_lookup(data, lookup_path)

# allow brackets for specifying environment names
VALID_SHORTCUT_CHARS = string.ascii_letters + string.digits + "[]_."
def _substitute_shortcuts(s, body_type):
    assert s.startswith('@')
    original = s
    s = s[len('@'):]
    if not s:
        raise common.ToughNoodles("No key found after @")
    if s[0] not in VALID_SHORTCUT_CHARS:
        raise common.ToughNoodles("Invalid char '{0}' found immediately after @"
                                  .format(s[0]))

    count = 0
    for c in s:
        if c not in VALID_SHORTCUT_CHARS:
            break
        count += 1

    # in case we have '@....poo', just compress the dots into a single dot
    key = s[:count].strip('.')
    if key.startswith('['):
        val = _lookup_item_from_environment(key)
    elif '.' in key:
        key = 'last.response.' + key
        val = _lookup_item_from_request(key, body_type)
    else:
        key = 'last.response.body.' + key
        val = _lookup_item_from_request(key, body_type)
    return val + s[count:]

def untemplate(s, withs={}, body_type='json', shortcuts=False):
    """Process the {{ }} sections of the template strings

    :param shortcuts: Look for shortcut syntax '@id' '@last.id' '@body.id'
    """
    withs = parse_withs(withs)
    while True:
        bounds = _find_double_braces(s)
        if not bounds:
            break
        a, b = bounds
        s = s[:a] + _substitute_braces(s[a:b], withs, body_type) + s[b:]

    if shortcuts:
        while True:
            i = s.find('@')
            if i < 0:
                break
            s = s[:i] + _substitute_shortcuts(s[i:], body_type)
    return s

def split_with(w):
    parts = w.split('=', 1)
    if len(parts) < 2:
        raise common.ToughNoodles("Bad with argument {0}".format(w))
    return parts

def parse_withs(withs):
    return {k: untemplate(v, shortcuts=True)
            for k, v in (split_with(w) for w in withs)}
