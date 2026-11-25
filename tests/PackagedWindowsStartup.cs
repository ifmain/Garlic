using System;
using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Threading;
class Test {
 [STAThread] static int Main(string[] args) {
  var a=new Application();
  var pkg=Assembly.LoadFrom(Path.Combine(AppDomain.CurrentDomain.BaseDirectory,"Garlic.exe"));
  var w=(Window)pkg.GetType("GarlicApp").GetMethod("CreateWindow").Invoke(null,new object[]{args[0]});
  bool done=false;
  w.Left=-20000;w.Top=-20000;w.ShowInTaskbar=false;
  var until=DateTime.UtcNow.AddSeconds(40);
  var t=new DispatcherTimer{Interval=TimeSpan.FromMilliseconds(200)};
  t.Tick+=delegate {
   string text=((TextBlock)w.FindName("Message")).Text;
   bool enabled=((Button)w.FindName("Login")).IsEnabled;

   if(enabled && w.Title=="Garlic Alpha" && !text.Contains("модуль")) done=true;
   if(done || DateTime.UtcNow>until){Console.WriteLine("UI "+args[0]+" title="+w.Title+" login="+enabled+" expected_state="+done);t.Stop();w.Close();}
  };t.Start();a.Run(w);return done?0:1;
 }
}
