from setuptools import setup

setup(name='spag',
      version='0.0.1',
      description='The best command line HTTP client ever',
      url='http://github.com/pglbutt/noodles',
      author='pglbutt',
      author_email='pglbutt@pglbutt.com',
      license='MIT',
      packages=['spag'],
      install_requires=[
        'Click',
      ],
      entry_points={
        'console_scripts': [
            'spag = spag.cli:cli'
        ]
      },
      zip_safe=False)
