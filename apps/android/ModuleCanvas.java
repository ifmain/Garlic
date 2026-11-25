package org.garlic.client;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.os.SystemClock;
import android.view.MotionEvent;
import android.view.View;
import android.widget.Toast;

public final class ModuleCanvas extends View {
    static { System.loadLibrary("garlic_internal_network"); }
    static native int open(String code);
    static native int[] tick(int x,int y,int pressed,int elapsed);
    static native void close();
    final Paint paint=new Paint();
    int x,y,pressed;
    boolean running;
    long previous;
    ModuleCanvas(Context c,String code) {
        super(c);
        running=open(code)!=0;
        if(!running)throw new IllegalArgumentException("Invalid canvas module");
        previous=SystemClock.uptimeMillis();
    }
    @Override public boolean onTouchEvent(MotionEvent e) {
        x=(int)(320*e.getX()/Math.max(1,getWidth())); y=(int)(240*e.getY()/Math.max(1,getHeight()));
        pressed=(e.getActionMasked()==MotionEvent.ACTION_UP || e.getActionMasked()==MotionEvent.ACTION_CANCEL)?0:1;
        getParent().requestDisallowInterceptTouchEvent(pressed!=0);return true;
    }
    @Override protected void onDraw(Canvas c){
        c.drawColor(0xff181818);
        if(!running)return;
        long now=SystemClock.uptimeMillis();
        int[] rects=tick(x,y,pressed,(int)Math.min(100,now-previous));previous=now;
        if(rects==null){running=false;Toast.makeText(getContext(),"Модуль остановлен",Toast.LENGTH_SHORT).show();return;}
        c.save();c.scale(getWidth()/320f,getHeight()/240f);
        for(int i=0;i<rects.length;i+=5){paint.setColor(rects[i+4]|0xff000000);c.drawRect(rects[i],rects[i+1],(float)rects[i]+rects[i+2],(float)rects[i+1]+rects[i+3],paint);}
        c.restore();postInvalidateDelayed(33);
    }
    @Override protected void onDetachedFromWindow(){super.onDetachedFromWindow();if(running)close();running=false;}
}
