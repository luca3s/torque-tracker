def mono_signal_pan():
    minus_3_decibels = 0.708
    for x in range(-32, 33):
        pan_pow = pow(x/32, 2)
        less = minus_3_decibels - pan_pow * minus_3_decibels
        more = minus_3_decibels + pan_pow * (1 - minus_3_decibels)

        if x > 0:
            print("(" + str(less) + ", " + str(more) + "), // " + str(x))
        else:
            print("(" + str(more) + ", " + str(less) + "), // " + str(x))

mono_signal_pan()