using System;
using System.IO;
using System.Text;
using System.Collections.Generic;
using System.Web.Script.Serialization;
using System.Windows;
using System.Windows.Controls;
using Microsoft.Win32;

internal static class PluginFiles
{
    const string DataFormat = "garlic-plugin-data/1";
    static Dictionary<string,object> Defaults() {
        return new Dictionary<string,object> { {"format", DataFormat}, {"id","dev-connection"},
            {"enabled", false}, {"interval_seconds",5}, {"ingress_penalty_ms",0} };
    }
    internal static Dictionary<string,object> Parse(string json, bool package) {
        var v = new JavaScriptSerializer { MaxJsonLength = 8192, RecursionLimit = 4 }
            .Deserialize<Dictionary<string,object>>(json);
        if (v == null || v.Count != (package ? 4 : 5) || !Equals(v["format"],package ? "gpf/1" : DataFormat) ||
            !Equals(v["id"],"dev-connection") || (!package && !(v["enabled"] is bool)) ||
            !(v["interval_seconds"] is int) || !(v["ingress_penalty_ms"] is int) ||
            (int)v["interval_seconds"] < 3 || (int)v["interval_seconds"] > 60 ||
            (int)v["ingress_penalty_ms"] < 0 || (int)v["ingress_penalty_ms"] > 250)
            throw new InvalidDataException();
        v["format"] = DataFormat;
        if(package) v["enabled"] = true;
        return v;
    }
    static string Read(string path) {
        using(var input=File.OpenRead(path)) {
            if(input.Length>8192) throw new InvalidDataException();
            using(var reader=new StreamReader(input,new UTF8Encoding(false,true))) return reader.ReadToEnd();
        }
    }
    internal static void Save(string path, Dictionary<string,object> value) {
        string dir=Path.GetDirectoryName(path);
        Directory.CreateDirectory(dir);
        string tmp=Path.Combine(dir,Guid.NewGuid().ToString("N") + ".tmp");
        try {
            File.WriteAllText(tmp,new JavaScriptSerializer().Serialize(value),new UTF8Encoding(false));
            if(File.Exists(path)) File.Replace(tmp,path,null); else File.Move(tmp,path);
        } finally { if(File.Exists(tmp)) File.Delete(tmp); }
    }
    static void Item(ContextMenu menu,string label,Action run,Action<string> status) {
        var item=new MenuItem {Header=label};
        item.Click += delegate { try { run(); } catch(Exception) { status("Ошибка файла плагина"); } };
        menu.Items.Add(item);
    }
    internal static void Show(Window owner,Button button,string config,Action<string> status) {
        string path=Path.ChangeExtension(config,"plugin-data");
        var menu=new ContextMenu();
        Func<Dictionary<string,object>> load=()=> File.Exists(path) ? Parse(Read(path),false) : Defaults();
        try {
            bool enabled=(bool)load()["enabled"];
            Item(menu,"Dev Connection · "+(enabled?"выключить":"включить"),delegate {
                var v=load(); v["enabled"]=!(bool)v["enabled"]; Save(path,v); status("Применится при следующем подключении");
            },status);
        } catch(Exception) { status("Настройки повреждены. Можно импортировать сохранённые данные."); }
        foreach(bool manifest in new[]{true,false}) {
            bool package=manifest;
            Item(menu, package ? "Установить GPF" : "Импорт данных (JSON)",delegate {
                var dialog=new OpenFileDialog { Filter=package?"Garlic Plugin Format|*.gpf":"JSON|*.json" };
                if(dialog.ShowDialog(owner)!=true) return;
                Save(path,Parse(Read(dialog.FileName),package)); status("Применится при следующем подключении");
            },status);
        }
        Item(menu,"Экспорт данных (JSON)",delegate {
            var dialog=new SaveFileDialog { Filter="JSON|*.json",FileName="dev-connection.json" };
            if(dialog.ShowDialog(owner)==true) { Save(dialog.FileName,load()); status("Данные сохранены"); }
        },status);
        menu.PlacementTarget=button; menu.IsOpen=true;
    }
}
