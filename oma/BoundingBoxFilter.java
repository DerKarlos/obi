package de.kumakyoo.omalibjava;

public class BoundingBoxFilter extends Filter
{
    protected BoundingBox bounds;

    protected BoundingBox cb;

    public BoundingBoxFilter(BoundingBox bounds)
    {
        this.bounds = bounds;
    }

    public BoundingBoxFilter(int minlon, int minlat, int maxlon, int maxlat)
    {
        this.bounds = new BoundingBox(minlon,minlat,maxlon,maxlat);
    }

    public BoundingBoxFilter(double minlon, double minlat, double maxlon, double maxlat)
    {
        this.bounds = new BoundingBox(minlon,minlat,maxlon,maxlat);
    }

    public boolean needsChunk(byte type, BoundingBox b)
    {
        cb = b;
        return bounds.intersects(b);
    }

    public boolean keep(Element e)
    {
        return e.isInside(bounds);
    }

    public boolean countable()
    {
        return bounds.contains(cb);
    }
}
