from os.path import dirname, realpath, join as path_join
from setuptools import setup, find_packages

from rsdbpy import __name__, __version__


package = __name__
version = __version__


# def valid_requirement(line):
#     if not line:
#         return False
#     else:
#         ch = line[0]
#         return ch not in ('#', '-')


# def parse_requirements(filename):
#     """ 解析依赖库 """
#     root = dirname(realpath(__file__))
#     line_iter = (line.strip() for line in open(path_join(root, filename)))
#     return [line for line in line_iter if valid_requirement(line)]


setup(
    name=package,
    version=version,
    description='rsdbpy',
    author='hepeng',
    author_email='dbt8625@163.com',
    include_package_data=True,
    url='aha:/home/alex/.data/gitrepo/rsdb.git',
    packages=find_packages(exclude=['tests', 'examples']),
    install_requires=[],  # parse_requirements('requirements.txt'),
    entry_points={
        'console_scripts': [
            # f'{package}-web = catalyst.webservice.entry:start_webapp',
            # f'{package}-version = catalyst.tools.version:show_version',
            # f'{package}-rs = catalyst.tools.results:show_results',
            # f'{package}-worker = catalyst.tools.worker:start',
            # f'{package}-beat = catalyst.tools.beat:start',
        ],
    },
)