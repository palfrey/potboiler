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
    next_port = 6432
    def __init__(self, name):
        self.base_port = Postgres.next_port
        Postgres.next_port +=1
        self.name = name
    def service(self):
        ret = OrderedDict()
        ret["image"] = "postgres"
        ret["environment"] = {"POSTGRES_PASSWORD":"mysecretpassword"}
        ret["ports"] = ["%d:5432"%self.base_port]
        return ret
    def db_url(self):
        return "postgres://postgres:mysecretpassword@postgres:5432"

class LocallyBuilt:
    def build_context(self, kind):
        ret = OrderedDict()
        ret["build"] = {
            "context": ".", 
            "dockerfile": "%s/Dockerfile" % kind
        }
        ret["image"] = "potboiler/%s:latest" % kind
        ret["volumes"] = [".:/code:cached"]
        return ret

class Core(LocallyBuilt):
    def __init__(self, name, index, postgres):
        self.base_port = 8000 + index*100
        self.name = name
        self.postgres = postgres
    def service(self):
        ret = self.build_context("core")
        ret["environment"] = {"DATABASE_URL": self.postgres.db_url()}
        ret["ports"] = ["%d:8000"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name]
        return ret
    def log_url(self):
        return "http://core:8000/log"

class KV(LocallyBuilt):
    def __init__(self, name, index, postgres, core):
        self.base_port = 8001 + index*100
        self.name = name
        self.postgres = postgres
        self.core = core
    def service(self):
        ret = self.build_context("kv")
        ret["environment"] = {
            "DATABASE_URL": self.postgres.db_url(),
            "SERVER_URL": self.core.log_url(),
            "HOST": self.name
        }
        ret["ports"] = ["%d:8001"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name, "%s:core"%self.core.name]
        return ret
    def base_url(self):
        return "http://kv:8001/kv"

class Pigtail(LocallyBuilt):
    def __init__(self, name, index, postgres, core):
        self.base_port = 8003 + index*100
        self.name = name
        self.postgres = postgres
        self.core = core
    def service(self):
        ret = self.build_context("pigtail")
        ret["environment"] = {
            "DATABASE_URL": self.postgres.db_url(),
            "SERVER_URL": self.core.log_url(),
            "HOST": self.name
        }
        ret["ports"] = ["%d:8000"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name, "%s:core"%self.core.name]
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
        ret["volumes"] = ["./kv-browser/:/code:cached"]
        ret["environment"] = {"DATABASE_URL": self.postgres.db_url()}
        ret["ports"] = ["%d:5000"%self.base_port]
        ret["links"] = ["%s:postgres"%self.postgres.name]
        return ret

class Correspondence:
    def __init__(self, name, index, kv):
        self.base_port = 8004 + index*100
        self.name = name
        self.kv = kv
    def service(self):
        ret = OrderedDict()
        ret["build"] = "../correspondence"
        ret["image"] = "potboiler/correspondence:latest"
        ret["volumes"] = ["../correspondence/:/code:cached"]
        ret["environment"] = {"SERVER_URL": self.kv.base_url()}
        ret["ports"] = ["%d:5000"%self.base_port]
        ret["links"] = ["%s:kv"%self.kv.name]
        ret["command"] = "bash -c \"./wait-for-it.sh kv:8001 -s -- flask run --host=0.0.0.0\""
        return ret

parser = argparse.ArgumentParser()
parser.add_argument('--component', action='append', default=[], dest="components", choices=["kv", "pigtail", "correspondence"])
parser.add_argument('count', type=int)
args = parser.parse_args()

services = []
for index in range(args.count):
    postgres = Postgres("postgres-core%d"%index)
    services.append(postgres)
    core = Core("core%d"%index, index, postgres)
    services.append(core)

    if args.components == [] or "kv" in args.components or "correspondence" in args.components:
        postgres = Postgres("postgres-kv%d"%index)
        services.append(postgres)
        kv = KV("kv%d"%index, index, postgres, core)
        services.append(kv)
        services.append(KVBrowser("kv-browser%d"%index, index, postgres))
    if args.components == [] or "pigtail" in args.components:
        postgres = Postgres("postgres-pigtail%d"%index)
        services.append(postgres)
        services.append(Pigtail("pigtail%d"%index, index, postgres, core))
    if "correspondence" in args.components:
        services.append(Correspondence("correspondence%d"%index, index, kv))

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
