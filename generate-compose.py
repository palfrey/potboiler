import yaml
from collections import OrderedDict
import argparse

compose = OrderedDict()
compose["version"] = "2"
compose["services"] = OrderedDict()

def extend(od, kind):
    od["extends"] = OrderedDict()
    od["extends"]["file"] = "common.yml"
    od["extends"]["service"] = kind

class Postgres:
    def __init__(self, name, index, additional=0):
        self.base_port = 6432 + index*1000 + additional
        self.name = name
    def service(self):
        ret = OrderedDict()
        ret["image"] = "postgres"
        ret["environment"] = {"POSTGRES_PASSWORD":"mysecretpassword"}
        ret["ports"] = ["%d:5432"%self.base_port]
        return ret
    def db_url(self):
        return "postgres://postgres:mysecretpassword@postgres:5432"

class Core:
    def __init__(self, name, index, postgres):
        self.base_port = 8000 + index*100
        self.name = name
        self.postgres = postgres
    def service(self):
        ret = OrderedDict()
        ret["build"] = {
            "context": ".", 
            "dockerfile": "core/Dockerfile"
        }
        ret["image"] = "potboiler/core:latest"
        ret["volumes"] = [".:/code"]
        ret["environment"] = {"DATABASE_URL": self.postgres.db_url()}
        ret["ports"] = ["%d:8000"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name]
        return ret
    def log_url(self):
        return "http://core:8000/log"

class KV:
    def __init__(self, name, index, postgres, core):
        self.base_port = 8001 + index*100
        self.name = name
        self.postgres = postgres
        self.core = core
    def service(self):
        ret = OrderedDict()
        ret["build"] = {
            "context": ".",
            "dockerfile": "kv/Dockerfile"
        }
        ret["image"] = "potboiler/kv:latest"
        ret["volumes"] = [".:/code"]
        ret["environment"] = {
            "DATABASE_URL": self.postgres.db_url(),
            "SERVER_URL": self.core.log_url()
        }
        ret["ports"] = ["%d:8001"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name, "%s:core"%self.core.name]
        ret["environment"] = ["HOST=%s" % self.name]
        return ret

class Pigtail:
    def __init__(self, name, index, postgres, core):
        self.base_port = 8003 + index*100
        self.name = name
        self.postgres = postgres
        self.core = core
    def service(self):
        ret = OrderedDict()
        ret["build"] = {
            "context": ".",
            "dockerfile": "pigtail/Dockerfile"
        }
        ret["image"] = "potboiler/pigtail:latest"
        ret["volumes"] = [".:/code"]
        ret["environment"] = {
            "DATABASE_URL": self.postgres.db_url(),
            "SERVER_URL": self.core.log_url()
        }
        ret["ports"] = ["%d:8000"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name, "%s:core"%self.core.name]
        ret["environment"] = ["HOST=%s" % self.name]
        return ret

class KVBrowser:
    def __init__(self, name, index, postgres):
        self.base_port = 8002 + index*100
        self.name = name
        self.postgres = postgres
    def service(self):
        ret = OrderedDict()
        ret["build"] = "kv-browser"
        ret["image"] = "potboiler/kv-browser:latest"
        ret["volumes"] = ["./kv-browser/:/code"]
        ret["environment"] = {"DATABASE_URL": self.postgres.db_url()}
        ret["ports"] = ["%d:5000"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name]
        return ret

parser = argparse.ArgumentParser()
parser.add_argument('--component', action='append', default=[], dest="components", choices=["kv", "pigtail"])
parser.add_argument('count', type=int)
args = parser.parse_args()

services = []
for index in range(args.count):
    postgres = Postgres("postgres-core%d"%index, index, 0)
    services.append(postgres)
    core = Core("core%d"%index, index, postgres)
    services.append(core)

    if args.components == [] or "kv" in args.components:
        postgres = Postgres("postgres-kv%d"%index, index, 1)
        services.append(postgres)
        services.append(KV("kv%d"%index, index, postgres, core))
        services.append(KVBrowser("kv-browser%d"%index, index, postgres))
    if args.components == [] or "pigtail" in args.components:
        postgres = Postgres("postgres-pigtail%d"%index, index, 2)
        services.append(postgres)
        services.append(Pigtail("pigtail%d"%index, index, postgres, core))

for service in services:
    compose["services"][service.name] = service.service()

# from http://blog.elsdoerfer.name/2012/07/26/make-pyyaml-output-an-ordereddict/
def represent_odict(dump, tag, mapping, flow_style=None):
    """Like BaseRepresenter.represent_mapping, but does not issue the sort().
    """
    value = []
    node = yaml.MappingNode(tag, value, flow_style=flow_style)
    if dump.alias_key is not None:
        dump.represented_objects[dump.alias_key] = node
    best_style = True
    if hasattr(mapping, 'items'):
        mapping = mapping.items()
    for item_key, item_value in mapping:
        node_key = dump.represent_data(item_key)
        node_value = dump.represent_data(item_value)
        if not (isinstance(node_key, yaml.ScalarNode) and not node_key.style):
            best_style = False
        if not (isinstance(node_value, yaml.ScalarNode) and not node_value.style):
            best_style = False
        value.append((node_key, node_value))
    if flow_style is None:
        if dump.default_flow_style is not None:
            node.flow_style = dump.default_flow_style
        else:
            node.flow_style = best_style
    return node

yaml.SafeDumper.add_representer(OrderedDict,
    lambda dumper, value: represent_odict(dumper, u'tag:yaml.org,2002:map', value))
print yaml.safe_dump(compose, default_flow_style=False)
