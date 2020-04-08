
def bit(x, z):
    return 1 << (x + z*11)

def in_bounds(x, z):
    distance_to_5 = abs(z - 5)
    if z > 5:
        if x >= distance_to_5:
            return True
    elif z < 5:
        if x < (11 - distance_to_5):
            return True
    else:
        return True
    return False

def valid_fields():
    valid = 0
    for x in range(11):
        for z in range(11):
            if (in_bounds(x, z)):
                valid |= bit(x, z)
    return valid

def print_val(label, val, gen_code=False):
    if (gen_code):
        print(f"pub const {label}: u128 = {val};")
    else:
        print(label+":")
        print("\t"+f"{val}")
        print("\t"+bin(val))

def mask_shift_noWe():
    val = 0
    # left edge
    for z in range(5, 11):
        x = (z - 5)
        val |= bit(x, z)
        
    # top row
    for x in range(5,11):
        val |= bit(x, z=10)
    return val

def mask_shift_noEa():
    val = 0
    # right edge
    for z in range(5, 11):
        val |= bit(10, z)
        
    # top row
    for x in range(5,11):
        val |= bit(x, z=10)
    return val

def mask_shift_soWe():
    val = 0
    # left edge
    for z in range(0, 6):
        val |= bit(0, z)
        
    # bottom row
    for x in range(5,11):
        val |= bit(x, z=0)
    return val

def mask_shift_soEa():
    val = 0
    # right edge
    for z in range(0, 6):
        val |= bit(5 + z, z)
        
    # bottom row
    for x in range(5,11):
        val |= bit(x, z=0)
    return val

def mask_shift_east():
    val = 0
    # top right edge
    for z in range(5, 11):
        val |= bit(10, z)
    # bottom right edge
    for z in range(0, 6):
        val |= bit(5 + z, z)
    return val

def mask_shift_unsafe_east():
    val = 0
    # top right edge
    for z in range(5, 11):
        val |= bit(10, z)
    return val

def mask_shift_unsafe_west():
    val = 0
    # bottom left edge
    for z in range(0, 6):
        val |= bit(0, z)
    return val

def mask_shift_west():
    val = 0
    # top left edge
    for z in range(5, 11):
        x = (z - 5)
        val |= bit(x, z)
    # bottom left edge
    for z in range(0, 6):
        val |= bit(0, z)
    return val

def other_direction(val):
    if val is 'west':
        return 'east'
    if val is 'east':
        return 'west'
    if val is 'nowe':
        return 'soea'
    if val is 'soea':
        return 'nowe'
    if val is 'sowe':
        return 'noea'
    if val is 'noea':
        return 'sowe'
    print("ERROR")

def gen_from_to_const(src, dest):
    first = other_direction(src)
    second = dest
    name = f"SHIFT_{src.upper()}_TO_{dest.upper()}_MASK"
    calculation = f"super::shift_{first}(SHIFT_{second.upper()}_MASK) | SHIFT_{first.upper()}_MASK"
    return f"pub const {name}: u128 = {calculation};"

def gen_from_to(src, dest):
    first = other_direction(src)
    ret = ""
    ret += "#[inline(always)]\n"
    ret += f"pub const fn shift_{src}_to_{dest}(bitboard: u128) -> u128 {'{'}\n"
    shift = shifts[first] + shifts[dest]
    if shift < 0:
        shift_part = f">> {shift}"
    else:
        shift_part = f"<< {shift}"
    ret += f"\treturn (bitboard & !constants::SHIFT_{src.upper()}_TO_{dest.upper()}_MASK) {shift_part};\n"
    ret += '}'
    return ret

directions = ['west', 'east', 'noea', 'nowe', 'soea', 'sowe']
shifts = {'west': -1, 'east': 1, 'noea': 12, 'sowe': -12, 'nowe': 11, 'soea': -11}

def main():
    gen_code = True
    print_val("VALID_FIELDS", valid_fields(), gen_code)
    print_val("SHIFT_NOWE_MASK", mask_shift_noWe(), gen_code)
    print_val("SHIFT_NOEA_MASK", mask_shift_noEa(), gen_code)
    print_val("SHIFT_SOWE_MASK", mask_shift_soWe(), gen_code)
    print_val("SHIFT_SOEA_MASK", mask_shift_soEa(), gen_code)
    print_val("SHIFT_EAST_MASK", mask_shift_east(), gen_code)
    print_val("SHIFT_WEST_MASK", mask_shift_west(), gen_code)
    print_val("SHIFT_EAST_UNSAFE_MASK", mask_shift_unsafe_east(), gen_code)
    print_val("SHIFT_WEST_UNSAFE_MASK", mask_shift_unsafe_west(), gen_code)
    for src in directions:
        for dest in directions:
            if not src is dest:
                print(gen_from_to_const(src, dest))
    for src in directions:
        for dest in directions:
            if not src is dest:
                print(gen_from_to(src, dest))

if __name__ == "__main__":
    main()