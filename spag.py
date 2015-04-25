#!/usr/bin/env python

import click
import os
import sys

class ToughNoodles(Exception):
    pass


class Endpoint(object):

    ENDPOINT_FILE = '.spag.endpoint'
    @classmethod
    def get(cls):
        if os.path.exists(cls.ENDPOINT_FILE):
            with click.open_file(cls.ENDPOINT_FILE, 'r') as f:
                return f.read()
        else:
            raise ToughNoodles("Endpoint not set")

    @classmethod
    def set(cls, endpoint):
        with click.open_file(cls.ENDPOINT_FILE, 'w') as f:
            f.write(endpoint)

    @classmethod
    def clear(cls):
        if os.path.exists(cls.ENDPOINT_FILE):
            os.remove(cls.ENDPOINT_FILE)


@click.command()
@click.argument('endpoint', default=None, required=False)
@click.option('--die', is_flag=True, default=False, required=False,
              help='Clear the endpoint')
def spag(die, endpoint=None):
    if die:
        Endpoint.clear()
        return

    try:
        if not endpoint:
            endpoint = Endpoint.get()
        else:
            Endpoint.set(endpoint)
        click.echo(endpoint)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)


if __name__ == '__main__':
    spag()
