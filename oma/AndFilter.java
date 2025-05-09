//package de.kumakyoo.omalibjava;

public class AndFilter extends Filter {

    private Filter[] fs;

    public AndFilter(Filter... fs) {
        this.fs = fs;
    }

    public boolean needsChunk(byte type, BoundingBox b) {
        for (Filter f : fs) if (!f.needsChunk(type, b)) return false;
        return true;
    }

    public boolean needsBlock(String key) {
        for (Filter f : fs) if (!f.needsBlock(key)) return false;
        return true;
    }

    public boolean needsSlice(String value) {
        for (Filter f : fs) if (!f.needsSlice(value)) return false;
        return true;
    }

    public boolean keep(Element e) {
        for (Filter f : fs) if (!f.keep(e)) return false;
        return true;
    }

    public boolean countable() {
        for (Filter f : fs) if (!f.countable()) return false;
        return true;
    }
}
