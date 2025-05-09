package de.kumakyoo.omalibjava;

import java.io.*;

public class PolygonFilter extends BoundingBoxFilter
{
    private Polygon poly;

    public PolygonFilter(String filename) throws IOException
    {
        super(null);
        poly = new Polygon(filename);
        bounds = poly.getBoundingBox();
    }

    public PolygonFilter(Polygon poly)
    {
        super(null);
        this.poly = poly;
        bounds = poly.getBoundingBox();
    }

    public boolean needsChunk(byte type, BoundingBox b)
    {
        cb = b;
        return bounds.intersects(b);
    }

    public boolean keep(Element e)
    {
        if (!super.keep(e)) return false;
        return e.isInside(poly);
    }

    public boolean countable()
    {
        return false;
    }
}
