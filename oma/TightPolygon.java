package de.kumakyoo.omalibjava;

import java.io.*;

public class TightPolygon extends Polygon implements Container
{
    public TightPolygon(String filename) throws IOException
    {
        super(filename);
    }

    public TightPolygon(String filename, int stripeSize) throws IOException
    {
        super(filename,stripeSize);
    }

    public TightPolygon(OmaReader r, Filter f) throws IOException
    {
        super(r,f);
    }

    public TightPolygon(OmaReader r, Filter f, int stripeSize) throws IOException
    {
        super(r,f,stripeSize);
    }

    public TightPolygon(Area[] areas)
    {
        super(areas);
    }

    public TightPolygon(Area[] areas, int stripeSize)
    {
        super(areas,stripeSize);
    }

    public TightPolygon(Polygon p)
    {
        super(p);
    }

    public boolean contains(int[] lon, int[] lat)
    {
        for (int i=0;i<lon.length;i++)
            if (!contains(lon[i],lat[i])) return false;
        return true;
    }

    public boolean contains(BoundingBox b)
    {
        return contains(b.minlon,b.minlat)
            && contains(b.minlon,b.maxlat)
            && contains(b.maxlon,b.minlat)
            && contains(b.maxlon,b.maxlat);
    }
}
