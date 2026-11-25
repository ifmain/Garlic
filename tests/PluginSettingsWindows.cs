using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Diagnostics;
using System.Reflection;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Threading;

class PluginSettingsWindows {
    static IEnumerable<DependencyObject> Descendants(DependencyObject node) {
        foreach(object child in LogicalTreeHelper.GetChildren(node)) {
            var element=child as DependencyObject;
            if(element==null)continue;
            yield return element;
            foreach(var nested in Descendants(element))yield return nested;
        }
    }
    static Button Button(Window w,string text){
        return Descendants(w).OfType<Button>().Single(b=>Equals(b.Content,text));
    }
    static void Click(Button b){b.RaiseEvent(new RoutedEventArgs(System.Windows.Controls.Primitives.ButtonBase.ClickEvent));}

    // An isolated profile drives the actual packaged WPF settings editor and its test callback.
    [STAThread] static int Main(string[] args) {
        string root=Path.Combine(Path.GetTempPath(),Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(root);
        try {
            var type=Assembly.LoadFrom(args[0]).GetType("PluginFiles",true);
            string config=Path.Combine(root,"client.json");
            type.GetMethod("ImportPackage",BindingFlags.Static|BindingFlags.NonPublic).Invoke(null,
                new object[]{config,File.ReadAllText(args[1]),new Func<string,bool>(s=>true)});
            var protocols=new MenuItem();
            var fill=type.GetMethod("FillProtocols",BindingFlags.Static|BindingFlags.NonPublic);
            Action<string> fail=message=>{throw new Exception(message);};
            fill.Invoke(null,new object[]{protocols,config,fail});
            var slot=protocols.Items.OfType<MenuItem>().Single();
            if(!Equals(slot.Header,"Dev Connection") || !slot.IsChecked)throw new Exception("Missing enabled protocol slot");
            foreach(bool enabled in new[]{false,true}){
                slot.IsChecked=enabled;slot.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
                fill.Invoke(null,new object[]{protocols,config,fail});slot=protocols.Items.OfType<MenuItem>().Single();
                if(slot.IsChecked!=enabled)throw new Exception("Slot toggle not persisted");
            }
            System.Threading.SynchronizationContext.SetSynchronizationContext(new DispatcherSynchronizationContext());
            var w=new Window();NameScope.SetNameScope(w,new NameScope());var grid=new Grid();var auth=new Grid();w.Content=grid;grid.Children.Add(auth);w.RegisterName("Auth",auth);
            Func<string,string,string,Task> test=(id,kind,addr)=>Task.Run(()=>{
                var si=new ProcessStartInfo(args[2],"plugin-test "+id+" "+kind+" "+addr){UseShellExecute=false,CreateNoWindow=true};
                si.EnvironmentVariables["GARLIC_CLIENT_CONFIG"]=config;
                using(var proc=Process.Start(si)){proc.WaitForExit();if(proc.ExitCode!=0)throw new Exception("Probe failed");}
            });
            type.GetMethod("Show",BindingFlags.Static|BindingFlags.NonPublic).Invoke(null,new object[]{
                w,new Button(),config,new Action<string>(s=>{throw new Exception(s);}),test,
                new Func<string,string,string,Task>((id,url,k)=>{throw new Exception("Unexpected Garlic key");})});
            Click(Button(w,"⚙  Настройки"));Click(Button(w,"＋  Добавить подключение"));
            var picker=Descendants(w).OfType<ComboBox>().Single();
            if(picker.Items.Count!=7)throw new Exception("Missing protocols");
            picker.SelectedIndex=Array.IndexOf(new[]{"direct","ingress","http","https","socks4","socks5","ssh"},args[3]);
            var boxes=Descendants(w).OfType<TextBox>().ToArray();boxes[0].Text=args[4];boxes[1].Text="fixture";boxes[2].Text=Console.ReadLine();
            Descendants(w).OfType<PasswordBox>().Single().Password=Console.ReadLine();
            Click(Button(w,"Сохранить"));
            var probe=Button(w,"Протестировать");Click(probe);
            var until=DateTime.UtcNow.AddSeconds(20);
            while(!probe.IsEnabled && DateTime.UtcNow<until) {
                var frame=new DispatcherFrame();
                Dispatcher.CurrentDispatcher.BeginInvoke(DispatcherPriority.Background,new Action(()=>frame.Continue=false));
                Dispatcher.PushFrame(frame);System.Threading.Thread.Sleep(10);
            }
            if(!Equals(probe.Content,"● Работает"))throw new Exception(Convert.ToString(probe.Content));
            return 0;
        } finally { Directory.Delete(root,true); }
    }
}
