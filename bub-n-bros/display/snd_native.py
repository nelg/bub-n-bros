# Not a sound driver for Client.py: in builds with only the Rust client
# (display/modes.py, RUST_CLIENT_ONLY), the web launcher shows this
# choice for "the Rust client plays its own sound and music".

def imperror():
    return None


def htmloptionstext(nameval):
    return None


class Sound:
    def __init__(self, *args, **kwds):
        raise SystemExit("the 'native' sound mode is the Rust client's own sound")
