import lldb
import decimal

class RustDecimalProvider(object):
    """Print a rust_decimal::Decimal"""

    def __init__(self, valobj, internal_dict):
        self.valobj = valobj
        self.lo = self.valobj.GetChildMemberWithName('lo').GetValueAsUnsigned()
        self.mid = self.valobj.GetChildMemberWithName('mid').GetValueAsUnsigned()
        self.hi = self.valobj.GetChildMemberWithName('hi').GetValueAsUnsigned()
        self.flags = self.valobj.GetChildMemberWithName('flags').GetValueAsUnsigned()
        self.scale = (self.flags & 0x00FF0000) >> 16
        self.sign = self.flags >> 31

    def num_children(self):
        return 1

    def get_child_index(self, name):
        return 0

    def get_child_at_index(self, index):
        child_type = self.valobj.target.GetBasicType(lldb.eBasicTypeChar)
        byte_order = self.valobj.GetData().GetByteOrder()
        data = lldb.SBData.CreateDataFromCString(byte_order, child_type.GetByteSize(), self.build_decimal())
        return self.valobj.CreateValueFromData("Decimal", data, child_type.GetArrayType(data.GetByteSize()))

    def update(self):
        return True

    def has_children(self):
        return True

    def build_decimal(self):
        mantissa = decimal.Decimal(self.hi)
        shift = decimal.Decimal(4294967296)
        mantissa = (mantissa * shift) + decimal.Decimal(self.mid)
        mantissa = (mantissa * shift) + decimal.Decimal(self.lo)
        value = mantissa
        divisor = decimal.Decimal(10)
        for i in range(self.scale):
            value = value / divisor
        if self.sign > 0:
            value = value * -1
        return str(value)

def __lldb_init_module(debugger, dict):
    debugger.HandleCommand('type synthetic add -x "Decimal" --python-class decimal_printer.RustDecimalProvider -w RustDecimal')
