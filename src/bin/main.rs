use macrotest::def_box;

def_box! {
    aligned(8) class Box (unsigned int(32) boxtype = bla, unsigned int(8)[16] extended_type) {
        unsigned int(32) size;
        unsigned int(32) type = boxtype;        # rust_type: FourCC
        if (size==1) {
            unsigned int(64) largesize;
        } else if (size==0) {
            unsigned int(16) loopy;
            // box extends to end of file
        } else if (size == "henk") {
            signed int(64) henkie;
        } else {
            signed int(8) keessie;
        }
        if (boxtype=="uuid") {
            unsigned int(8)[16] usertype = extended_type;   # optional; rust_type: Uuid
        }
    }
}

def_box! {
    aligned(8) class FullBox(unsigned int(32) boxtype, unsigned int(8) v, bit(24) f) extends Box(boxtype) {
        unsigned int(8) version = v;
        bit(24) flags = f;
    }
}

fn main() {
}

