package de.kumakyoo.omalibjava;

import java.util.*;
import java.io.*;

public class Polygon implements Container
{
    private static final int DEFAULT_STRIPE_SIZE = 100000;

    protected Map<Integer,List<Line>> stripes;

    protected final int stripeSize;

    public Polygon(String filename) throws IOException
    {
        this(filename,DEFAULT_STRIPE_SIZE);
    }

    public Polygon(String filename, int stripeSize) throws IOException
    {
        this.stripeSize = stripeSize;
        readPolygon(filename);
    }

    public Polygon(OmaReader r, Filter f) throws IOException
    {
        this(r,f,DEFAULT_STRIPE_SIZE);
    }

    public Polygon(OmaReader r, Filter f, int stripeSize) throws IOException
    {
        this.stripeSize = stripeSize;
        queryPolygon(r,f);
    }

    public Polygon(Area[] areas)
    {
        this(areas,DEFAULT_STRIPE_SIZE);
    }

    public Polygon(Area[] areas, int stripeSize)
    {
        this.stripeSize = stripeSize;
        polygonFromAreas(areas);
    }

    public Polygon(Polygon tp)
    {
        this.stripeSize = tp.stripeSize;
        this.stripes = tp.stripes;
    }

    public BoundingBox getBoundingBox()
    {
        boolean first = true;
        long minlon = 0;
        long maxlon = 0;
        long minlat = 0;
        long maxlat = 0;
        for (List<Line> ll: stripes.values())
            for (Line l: ll)
            {
                if (first)
                {
                    minlon = maxlon = l.x1;
                    minlat = maxlat = l.y1;
                    first = false;
                }

                minlon = Math.min(minlon,Math.min(l.x1,l.x2));
                maxlon = Math.max(maxlon,Math.max(l.x1,l.x2));
                minlat = Math.min(minlat,Math.min(l.y1,l.y2));
                maxlat = Math.max(maxlat,Math.max(l.y1,l.y2));
            }

        return new BoundingBox((int)minlon,(int)minlat,(int)maxlon,(int)maxlat);
    }

    private List<Line> get(int lon, int lat)
    {
        int nr = lat/stripeSize;
        return stripes.get(nr);
    }

    public boolean contains(int lon, int lat)
    {
        int nr = lat/stripeSize;
        List<Line> ll = stripes.get(nr);

        if (ll==null) return false;

        boolean inside = false;

        for (Line l:ll)
        {
            if (l.x1>lon) break;
            if ((l.y1<=lat) != (lat<l.y2)) continue;
            if (l.x1 + (l.x2-l.x1)*(lat-l.y1)/(l.y2-l.y1)<lon)
                inside = !inside;
        }

        return inside;
    }

    public boolean contains(int[] lon, int[] lat)
    {
        for (int i=0;i<lon.length;i++)
            if (contains(lon[i],lat[i])) return true;
        return false;
    }

    public boolean contains(BoundingBox b)
    {
        return contains(b.minlon,b.minlat)
            || contains(b.minlon,b.maxlat)
            || contains(b.maxlon,b.minlat)
            || contains(b.maxlon,b.maxlat);
    }

    private void polygonFromAreas(Area[] areas)
    {
        stripes = new HashMap<>();

        for (Area a:areas)
            addArea(a);

        sortStripes();
    }

    private void queryPolygon(OmaReader r, Filter f) throws IOException
    {
        Filter save = r.getFilter();

        r.reset();
        r.setFilter(new AndFilter(f,new TypeFilter("A")));

        stripes = new HashMap<>();

        while (true)
        {
            Area a = (Area)r.next();
            if (a==null) break;

            addArea(a);
        }

        sortStripes();

        r.reset();
        r.setFilter(save);
    }

    private void addArea(Area a)
    {
        List<Point> poly = new ArrayList<>();
        for (int i=0;i<a.lon.length;i++)
            poly.add(new Point(a.lon[i],a.lat[i]));
        addStripes(poly);

        for (int j=0;j<a.holes_lon.length;j++)
        {
            poly = new ArrayList<>();
            for (int i=0;i<a.holes_lon[j].length;i++)
                poly.add(new Point(a.holes_lon[j][i],a.holes_lat[j][i]));
            addStripes(poly);
        }
    }

    private void readPolygon(String filename) throws IOException
    {
        BufferedReader r = new BufferedReader(new FileReader(filename));

        stripes = new HashMap<>();

        List<Point> poly = new ArrayList<>();

        r.readLine();
        while (true)
        {
            String line = r.readLine();
            if (line==null) break;
            if (line.length()==0) continue;

            if (line.charAt(0)==' ')
            {
                String[] tmp = line.trim().split(" ");
                poly.add(new Point(conv(tmp[0]),conv(tmp[1])));
            }
            else
            {
                if (poly.size()==0) continue;

                addStripes(poly);
                poly = new ArrayList<>();
            }
        }

        sortStripes();
    }

    private void addStripes(List<Point> poly)
    {
        for (int i=0;i<poly.size();i++)
        {
            Point a = poly.get(i);
            Point b = poly.get((i+1)%poly.size());
            if (a.y==b.y) continue;

            long top = Math.min(a.y,b.y);
            long bot = Math.max(a.y,b.y);

            int startseg = (int)(top/stripeSize);
            int stopseg = (int)(bot/stripeSize);

            if (b.x<a.x)
            {
                Point tmp = a;
                a = b;
                b = tmp;
            }

            for (int j=startseg;j<=stopseg;j++)
            {
                List<Line> ll = stripes.get(j);
                if (ll==null) ll = new ArrayList<>();
                ll.add(new Line(a.x,a.y,b.x,b.y));
                stripes.put(j,ll);
            }
        }
    }

    private void sortStripes()
    {
        for (int j: stripes.keySet())
        {
            List<Line> ll = stripes.get(j);
            Collections.sort(ll,(a, b) -> (int)(a.x1-b.x1));
            stripes.put(j,ll);
        }
    }

    private long conv(String s)
    {
        return (long)(Double.parseDouble(s)*1e7+0.5);
    }

    class Point
    {
        long x, y;

        public Point(long x, long y)
        {
            this.x = x;
            this.y = y;
        }
    }

    class Line
    {
        long x1, y1, x2, y2;

        public Line(long x1, long y1, long x2, long y2)
        {
            this.x1 = x1;
            this.y1 = y1;
            this.x2 = x2;
            this.y2 = y2;
        }
    }
}
