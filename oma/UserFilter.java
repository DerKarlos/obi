package de.kumakyoo.omalibjava;

public class UserFilter extends Filter
{
    private int uid;
    private String user;

    public UserFilter(int uid)
    {
        this.uid = uid;
    }

    public UserFilter(String user)
    {
        if (user==null) user="";
        this.user = user;
    }

    public boolean keep(Element e)
    {
        return user!=null?user.equals(e.user):(uid==e.uid);
    }

    public boolean countable()
    {
        return false;
    }
}
