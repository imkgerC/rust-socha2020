
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

def print_val(label, val):
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

def main():
    print_val("valid_fields", valid_fields())
    print_val("shift_noWe_mask", mask_shift_noWe())
    print_val("shift_noEa_mask", mask_shift_noEa())
    print_val("shift_soWe_mask", mask_shift_soWe())
    print_val("shift_soEa_mask", mask_shift_soEa())
    print_val("shift_east_mask", mask_shift_east())
    print_val("shift_west_mask", mask_shift_west())

if __name__ == "__main__":
    main()