using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Threading;

internal sealed class ModuleCanvas : FrameworkElement {
    [DllImport("garlic_internal_network.dll",CallingConvention=CallingConvention.Cdecl)]
    static extern int garlic_view_open(byte[] code,UIntPtr length);
    [DllImport("garlic_internal_network.dll",CallingConvention=CallingConvention.Cdecl)]
    static extern int garlic_view_tick(int x,int y,int down,int ms,[Out]int[] output,UIntPtr length);
    [DllImport("garlic_internal_network.dll",CallingConvention=CallingConvention.Cdecl)]
    static extern void garlic_view_close();
    readonly int[] rects=new int[640];
    readonly DispatcherTimer timer=new DispatcherTimer { Interval=TimeSpan.FromMilliseconds(33) };
    readonly System.Diagnostics.Stopwatch clock=new System.Diagnostics.Stopwatch();
    int count;
    public ModuleCanvas(string code) {
        Height=300;ClipToBounds=true;
        Loaded+=delegate {
            var bytes=Encoding.ASCII.GetBytes(code);
            if(garlic_view_open(bytes,(UIntPtr)bytes.Length)==0){ToolTip="Ошибка запуска модуля";return;}
            clock.Restart();timer.Start();
        };
        Unloaded+=delegate{timer.Stop();garlic_view_close();};
        timer.Tick+=delegate{
            var pos=Mouse.GetPosition(this);
            count=garlic_view_tick((int)(320*pos.X/Math.Max(1,ActualWidth)),(int)(240*pos.Y/Math.Max(1,ActualHeight)),
                IsMouseOver && Mouse.LeftButton==MouseButtonState.Pressed?1:0,(int)Math.Min(100,clock.ElapsedMilliseconds),rects,(UIntPtr)rects.Length);
            clock.Restart();if(count<0){count=0;timer.Stop();ToolTip="Модуль остановлен";}InvalidateVisual();
        };
    }
    protected override void OnRender(DrawingContext dc) {
        dc.DrawRectangle(Brushes.Black,null,new Rect(0,0,ActualWidth,ActualHeight));
        dc.PushTransform(new ScaleTransform(ActualWidth/320,ActualHeight/240));
        for(int i=0;i<count;i+=5){
            int rgb=rects[i+4];
            var brush=new SolidColorBrush(Color.FromRgb((byte)(rgb>>16),(byte)(rgb>>8),(byte)rgb));
            dc.DrawRectangle(brush,null,new Rect(rects[i],rects[i+1],Math.Max(0,rects[i+2]),Math.Max(0,rects[i+3])));
        }
        dc.Pop();
    }
}
