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

def postgres(index, additional=0):
    ret = OrderedDict()
    extend(ret, "postgres-base")
    base_port = 6432 + index*1000 + additional
    ret["ports"] = ["%d:5432"%base_port]
    return ret

def core(index):
    ret = OrderedDict()
    extend(ret, "core-base")
    base_port = 8000 + index*100
    ret["ports"] = ["%d:8000"%base_port]
    ret["links"] = ["postgres-core%d:postgres"%index]
    return ret

def kv(index):
    ret = OrderedDict()
    extend(ret, "kv-base")
    base_port = 8001 + index*100
    ret["ports"] = ["%d:8001"%base_port]
    ret["links"] = ["postgres-kv%d:postgres"%index, "core%d:core"%index]
    ret["environment"] = ["HOST=kv%d" % index]
    return ret

def pigtail(index):
    ret = OrderedDict()
    extend(ret, "pigtail-base")
    base_port = 8003 + index*100
    ret["ports"] = ["%d:8000"%base_port]
    ret["links"] = ["postgres-pigtail%d:postgres"%index, "core%d:core"%index]
    ret["environment"] = ["HOST=pigtail%d" % index]
    return ret

def kv_browser(index):
    ret = OrderedDict()
    extend(ret, "kv-browser-base")
    base_port = 8002 + index*100
    ret["ports"] = ["%d:5000"%base_port]
    ret["links"] = ["postgres-kv%d:postgres"%index]
    return ret

parser = argparse.ArgumentParser()
parser.add_argument('--component', action='append', default=[], dest="components", choices=["kv", "pigtail"])
parser.add_argument('count', type=int)
args = parser.parse_args()

for index in range(args.count):
    compose["services"]["core%d"%index] = core(index)
    compose["services"]["postgres-core%d"%index] = postgres(index, 0)
    if args.components == [] or "kv" in args.components:
        compose["services"]["kv%d"%index] = kv(index)
        compose["services"]["postgres-kv%d"%index] = postgres(index, 1)
        compose["services"]["kv-browser%d"%index] = kv_browser(index)
    if args.components == [] or "pigtail" in args.components:
        compose["services"]["pigtail%d"%index] = pigtail(index)
        compose["services"]["postgres-pigtail%d"%index] = postgres(index, 2)

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
