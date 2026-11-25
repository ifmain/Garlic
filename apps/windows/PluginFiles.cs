using System;
using System.IO;
using System.Linq;
using System.Collections;
using System.Windows.Media;
using System.Text;
using System.Collections.Generic;
using System.Web.Script.Serialization;
using System.Windows;
using System.Windows.Controls;
using Microsoft.Win32;

internal static class PluginFiles
{
    static bool Builtin(string k){return k=="direct"||k=="ingress";}
    static string Display(string k){return k=="direct"?"Garlic · напрямую":k=="ingress"?"Garlic · входная нода":k.ToUpperInvariant();}
    const string DataFormat = "garlic-plugin-data/1";
    internal static Dictionary<string,object> Parse(string json) {
        var v = new JavaScriptSerializer { MaxJsonLength = 8192, RecursionLimit = 4 }
            .Deserialize<Dictionary<string,object>>(json);
        if (v == null || v.Count != (v.ContainsKey("connections") ? 6 : 5) || !Equals(v["format"],DataFormat) ||
            !System.Text.RegularExpressions.Regex.IsMatch(v["id"] as string ?? "",@"\A[a-z0-9-]{1,48}\z") || !(v["enabled"] is bool) ||
            !(v["interval_seconds"] is int) || !(v["ingress_penalty_ms"] is int) ||
            (int)v["interval_seconds"] < 3 || (int)v["interval_seconds"] > 60 ||
            (int)v["ingress_penalty_ms"] < 0 || (int)v["ingress_penalty_ms"] > 250)
            throw new InvalidDataException();
        var rows=new List<Dictionary<string,object>>();
        if(v.ContainsKey("connections")){
            var items=v["connections"] as IList;
            if(items==null || items.Count>24)throw new InvalidDataException();
            foreach(var item in items){
                var row=item as Dictionary<string,object>;
                if(row==null || row.Count!=2 || !System.Text.RegularExpressions.Regex.IsMatch(row["kind"] as string ?? "",@"\A[a-z][a-z0-9-]{0,31}\z"))throw new InvalidDataException();
                row["address"]=Address(row["address"] as string,(string)row["kind"]);rows.Add(row);
            }
        }
        v["connections"]=rows;
        v["format"] = DataFormat;
        return v;
    }
    static Dictionary<string,object> Manifest(string path) {
        return new JavaScriptSerializer().Deserialize<Dictionary<string,object>>(Read(path));
    }
    internal static void Install(string path,string text) {
        var serializer=new JavaScriptSerializer {MaxJsonLength=16384,RecursionLimit=4};
        var env=serializer.Deserialize<Dictionary<string,string>>(text);
        var raw=Convert.FromBase64String(env["payload"]);
        if(!ReleaseClient.Authentic(raw,Convert.FromBase64String(env["signature"])))throw new InvalidDataException();
        var m=serializer.Deserialize<Dictionary<string,object>>(Encoding.UTF8.GetString(raw));
        if(!Equals(m["format"],"gpf/2") || ((string)m["name"]).Length>80 || ((string)m["wasm"]).Length>8192)throw new InvalidDataException();
        var opts=new Dictionary<string,object>{{"format",DataFormat},{"id",m["id"]},{"enabled",true},{"interval_seconds",m["interval_seconds"]},{"ingress_penalty_ms",m["ingress_penalty_ms"]}};
        var validated=Parse(serializer.Serialize(opts));
        if(File.Exists(path)){validated=Parse(Read(path));if(!Equals(validated["id"],m["id"]))throw new InvalidDataException();}
        Save(path,validated);Save(Path.ChangeExtension(path,"gpf-module"),m);
    }
    internal static void ImportSettings(string path,string module,string input) {
        var json=new JavaScriptSerializer{MaxJsonLength=8192,RecursionLimit=8};
        var document=json.Deserialize<Dictionary<string,object>>(input);
        object supplied;
        bool hasCredentials=document.TryGetValue("credentials",out supplied);
        document.Remove("credentials");
        var settings=Parse(json.Serialize(document));
        if(!Equals(settings["id"],Manifest(module)["id"]))throw new InvalidDataException();
        if(hasCredentials) {
            var creds=supplied as Dictionary<string,object>;
            if(creds==null)throw new InvalidDataException();
            string file=Path.ChangeExtension(path,"connector-login");
            var saved=File.Exists(file)?Manifest(file):new Dictionary<string,object>();
            var allowed=new HashSet<string>(((List<Dictionary<string,object>>)settings["connections"])
                .Where(row=>!Equals(row["kind"],"direct") && !Equals(row["kind"],"ingress")).Select(row=>(string)row["address"]));
            foreach(var item in creds) {
                var login=item.Value as Dictionary<string,object>;
                if(!allowed.Contains(item.Key) || login==null || login.Count!=3 ||
                   new[]{"user","password","pin"}.Any(k=>!login.ContainsKey(k) || !(login[k] is string)))throw new InvalidDataException();
                string password=(string)login["password"];
                if(password.Length>0)login["password"]="dpapi:"+Convert.ToBase64String(System.Security.Cryptography.ProtectedData.Protect(
                    Encoding.UTF8.GetBytes(password),null,System.Security.Cryptography.DataProtectionScope.CurrentUser)).TrimEnd('=').Replace('+','-').Replace('/','_');
                saved[item.Key]=login;
            }
            Save(file,saved);
        }
        Save(path,settings);
    }
    static string Library(string config){return Path.Combine(Path.GetDirectoryName(config),"plugins-v3");}
    static IEnumerable<string> Slots(string config) {
        var legacy=Path.ChangeExtension(config,"installed-gpf");
        if(File.Exists(Path.ChangeExtension(legacy,"gpf-module")))yield return legacy;
        if(Directory.Exists(Library(config)))foreach(string dir in Directory.GetDirectories(Library(config),"s-*").OrderBy(x=>x,StringComparer.Ordinal)){
            var state=Path.Combine(dir,"client.installed-gpf");if(File.Exists(Path.ChangeExtension(state,"gpf-module")))yield return state;
        }
    }
    internal static void FillProtocols(MenuItem menu,string config,Action<string> error) {
        menu.Items.Clear();
        foreach(string path in Slots(config))try{
            var m=Manifest(Path.ChangeExtension(path,"gpf-module"));
            if(!m.ContainsKey("capability")||!Equals(m["capability"],"network"))continue;
            var state=Parse(Read(path));
            if(!Equals(state["id"],m["id"]))continue;
            var item=new MenuItem{Header=(string)m["name"],IsCheckable=true,IsChecked=(bool)state["enabled"]};
            item.Click+=delegate{try{var current=Parse(Read(path));current["enabled"]=item.IsChecked;Save(path,current);}
                catch(Exception e){error(e.Message);}};
            menu.Items.Add(item);
        }catch(Exception e){error(e.Message);}
        menu.IsEnabled=menu.Items.Count>0;
    }
    internal static void ImportPackage(string config,string text,Func<string,bool> confirm) {
        var j=new JavaScriptSerializer{MaxJsonLength=16384,RecursionLimit=4};
        var env=j.Deserialize<Dictionary<string,string>>(text);
        if(!env.ContainsKey("author")){Install(Path.ChangeExtension(config,"installed-gpf"),text);return;}
        byte[] modulus=Convert.FromBase64String(env["author"]),raw=Convert.FromBase64String(env["payload"]);
        if(modulus.Length!=384 || (modulus[0]&128)==0)throw new InvalidDataException();
        using(var rsa=new System.Security.Cryptography.RSACryptoServiceProvider(new System.Security.Cryptography.CspParameters(24){Flags=System.Security.Cryptography.CspProviderFlags.CreateEphemeralKey})){
            rsa.ImportParameters(new System.Security.Cryptography.RSAParameters{Modulus=modulus,Exponent=new byte[]{1,0,1}});
            if(!rsa.VerifyData(raw,"SHA256",Convert.FromBase64String(env["signature"])))throw new InvalidDataException();
        }
        var m=j.Deserialize<Dictionary<string,object>>(new UTF8Encoding(false,true).GetString(raw));
        string id=(string)m["id"],name=(string)m["name"],cap=(string)m["capability"];
        if(!(Equals(m["format"],"gpf/3") || (Equals(m["format"],"gpf/4") && cap=="network" && Equals(m["wire_abi"],1))) || !(cap=="network"||cap=="canvas") || name.Length>80 || name.Length==0 ||
          !System.Text.RegularExpressions.Regex.IsMatch(id,@"\A[a-z0-9-]{1,48}\z") || ((string)m["wasm"]).Length>8192)throw new InvalidDataException();
        string state=Path.Combine(Library(config),"s-"+id,"client.installed-gpf"),module=Path.ChangeExtension(state,"gpf-module");
        string fingerprint;
        using(var hash=System.Security.Cryptography.SHA256.Create())fingerprint=BitConverter.ToString(hash.ComputeHash(modulus)).Replace("-","");
        if(File.Exists(module)&&!Equals(Manifest(module)["publisher"],fingerprint))throw new InvalidDataException("Publisher mismatch");
        if(!confirm(name+" ("+id+")\n"+(cap=="canvas"?"Экран и ввод на нём":Equals(m["format"],"gpf/4")?"Обработка трафика слота и данных входа выбранного подключения":"Выбор разрешённых подключений")+"\nSHA-256: "+fingerprint))return;
        if(!File.Exists(state))Save(state,new Dictionary<string,object>{{"id",id},{"format",DataFormat},{"enabled",true},{"connections",new object[0]},{"interval_seconds",5},{"ingress_penalty_ms",0}});
        m["publisher"]=fingerprint;Save(module,m);
    }
    static string Read(string path) {
        using(var input=File.OpenRead(path)) {
            if(input.Length>16384) throw new InvalidDataException();
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
    static string Address(string value,string kind="direct") {
        Uri u;
        if(value==null || value.Length>240 || !Uri.TryCreate(value,UriKind.Absolute,out u) ||
           u.Scheme!=(kind=="direct"||kind=="ingress"?"https":kind) || u.Host.Length==0 || u.UserInfo!="" || (u.AbsolutePath!="/" && u.AbsolutePath!="") ||
           u.Query!="" || u.Fragment!="" || u.Port==0) throw new InvalidDataException();
        return u.GetLeftPart(UriPartial.Authority);
    }
    internal static void Show(Window owner,Button button,string config,Action<string> status,Func<string,string,string,System.Threading.Tasks.Task> test,Func<string,string,string,System.Threading.Tasks.Task> storeKey) {
        string path=Path.ChangeExtension(config,"installed-gpf");
        var host=(Grid)((FrameworkElement)owner.FindName("Auth")).Parent;
        var previous=host.Children.OfType<ScrollViewer>().FirstOrDefault(x=>Equals(x.Tag,"plugins"));
        if(previous!=null)host.Children.Remove(previous);
        var panel=new StackPanel { MaxWidth=510,HorizontalAlignment=HorizontalAlignment.Stretch };
        var page=new ScrollViewer { Content=panel,Tag="plugins",Background=new SolidColorBrush(Color.FromRgb(24,24,24)),VerticalScrollBarVisibility=ScrollBarVisibility.Auto };
        host.Children.Add(page);
        bool details=false;
        Action refresh=null;
        Action<Panel,string,Action> action=(parent,title,click)=>{
            var b=new Button { Content=title,Margin=new Thickness(0,4,8,4),MinWidth=72 };
            b.Click+=delegate {try{click();}catch(Exception){status("Проверьте формат файла или адрес подключения");}}; parent.Children.Add(b);
        };
        Action<string,int> label=(title,size)=>panel.Children.Add(new TextBlock{Text=title,FontSize=size,TextWrapping=TextWrapping.Wrap,Margin=new Thickness(0,12,0,6)});
        refresh=delegate {
            panel.Children.Clear();
            if(!details) {
                action(panel,"‹  Назад",delegate{host.Children.Remove(page);});label("Плагины",28);
                var slots=Slots(config).ToList();if(slots.Count==0)label("Нет установленных плагинов",16);
                foreach(string item in slots){
                    Dictionary<string,object> m,data;
                    try {m=Manifest(Path.ChangeExtension(item,"gpf-module"));data=Parse(Read(item));}
                    catch {label("Ошибка чтения слота "+Path.GetFileName(Path.GetDirectoryName(item)),14);continue;}
                    label((string)m["name"],21);var bar=new WrapPanel();panel.Children.Add(bar);
                    var toggle=new CheckBox{Content="Включён",IsChecked=(bool)data["enabled"],VerticalAlignment=VerticalAlignment.Center};
                    toggle.Click+=delegate{try{data["enabled"]=toggle.IsChecked==true;Save(item,data);}catch{refresh();}};bar.Children.Add(toggle);
                    action(bar,"⚙  Настройки",delegate{path=item;details=true;refresh();});
                }
                action(panel,"Установить GPF",delegate{var d=new OpenFileDialog{Filter="Garlic Plugin Format|*.gpf"};
                    if(d.ShowDialog(owner)==true){ImportPackage(config,Read(d.FileName),msg=>MessageBox.Show(owner,msg,"Установить плагин?",MessageBoxButton.YesNo)==MessageBoxResult.Yes);refresh();}});
                return;
            }
            string module=Path.ChangeExtension(path,"gpf-module");var manifest=Manifest(module);
            action(panel,"‹  Плагины",delegate{details=false;refresh();});label((string)manifest["name"],28);
            Action import=delegate {
                var d=new OpenFileDialog{Filter="JSON|*.json"};
                if(d.ShowDialog(owner)==true && File.Exists(path)){ImportSettings(path,module,Read(d.FileName));refresh();}
            };
            Dictionary<string,object> v;
            try{v=Parse(Read(path));}catch{label("Настройки повреждены",16);action(panel,"Импорт JSON",import);return;}
            if(manifest.ContainsKey("capability") && Equals(manifest["capability"],"canvas")){
                if((bool)v["enabled"])panel.Children.Add(new ModuleCanvas((string)manifest["wasm"]));else label("Плагин выключен",16);
            }else{
            var rows=(List<Dictionary<string,object>>)v["connections"];
            label("Подключения",20);label("Изменения применятся при следующем подключении.",14);
            var transportList=manifest.ContainsKey("transports")?manifest["transports"] as IList:null;
            var kinds=new[]{"direct","ingress"}.Concat(transportList==null?Enumerable.Empty<string>():transportList.OfType<string>()).Distinct().ToArray();
            string secretPath=Path.ChangeExtension(path,"connector-login");
            var secrets=File.Exists(secretPath)?Manifest(secretPath):new Dictionary<string,object>();
            Action<int> editor=index=>{
                panel.Children.Clear();label(index<0?"Добавить подключение":"Подключение",26);
                label("Тип подключения",15);var kind=new ComboBox{ItemsSource=kinds.Select(Display).ToList(),SelectedIndex=index<0?0:Array.IndexOf(kinds,(string)rows[index]["kind"])};panel.Children.Add(kind);
                label("Адрес сервера",15);var addr=new TextBox{Text=index<0?"":(string)rows[index]["address"],ToolTip="https://server.example:8443"};panel.Children.Add(addr);
                label("Пользователь прокси / SSH",15);var user=new TextBox();panel.Children.Add(user);
                label("Отпечаток SSH (SHA256:…) или свой CA для HTTPS (DER в Base64)",15);var pin=new TextBox();panel.Children.Add(pin);
                if(index>=0 && secrets.ContainsKey((string)rows[index]["address"])){
                    var login=(Dictionary<string,object>)secrets[(string)rows[index]["address"]];user.Text=(string)login["user"];pin.Text=(string)login["pin"];
                }
                label("Ключ Garlic или пароль подключения",15); var key=new PasswordBox{ToolTip="Оставьте пустым, чтобы использовать сохранённый ключ"};panel.Children.Add(key);
                var submit=new Button{Content="Сохранить",Margin=new Thickness(0,8,0,4)};panel.Children.Add(submit);
                submit.Click+=async delegate{submit.IsEnabled=false;try{
                    string type=kinds[kind.SelectedIndex];string endpoint=Address(addr.Text.Trim(),type);
                    var item=new Dictionary<string,object>{{"kind",type},{"address",endpoint}};
                    if(index<0 && rows.Count>=24)throw new InvalidDataException();
                    if(kind.SelectedIndex>=2){
                        var oldLogin=secrets.ContainsKey(endpoint)?(Dictionary<string,object>)secrets[endpoint]:null;
                        secrets[endpoint]=new Dictionary<string,object>{{"user",user.Text},{"pin",pin.Text.Trim()},{"password",key.Password.Length>0?"dpapi:"+Convert.ToBase64String(System.Security.Cryptography.ProtectedData.Protect(Encoding.UTF8.GetBytes(key.Password),null,System.Security.Cryptography.DataProtectionScope.CurrentUser)).TrimEnd('=').Replace('+','-').Replace('/','_'):oldLogin!=null && Equals(oldLogin["user"],user.Text)?oldLogin["password"]:""}};
                        Save(secretPath,secrets);
                    }else if(key.Password.Length>0){await storeKey(manifest.ContainsKey("capability")?(string)manifest["id"]:null,(string)item["address"],key.Password);key.Clear();}
                    if(index<0){rows.Add(item);}else rows[index]=item;
                    Save(path,v);refresh();
                }catch{status("Не удалось сохранить подключение. Проверьте адрес и ключ.");}finally{submit.IsEnabled=true;}};
                if(index>=0)action(panel,"Удалить подключение",delegate{rows.RemoveAt(index);Save(path,v);refresh();});
                action(panel,"Отмена",refresh);
            };
            for(int i=0;i<rows.Count;i++){int index=i;action(panel,Display((string)rows[i]["kind"])+"   ·   "+rows[i]["address"],()=>editor(index));
                if(!Builtin((string)rows[i]["kind"])){
                    var probe=new Button{Content="Протестировать"};panel.Children.Add(probe);
                    probe.Click+=async delegate{probe.IsEnabled=false;probe.Content="Проверяем…";try{
                        await test((string)manifest["id"],(string)rows[index]["kind"],(string)rows[index]["address"]);
                        probe.Content="● Работает";probe.Foreground=Brushes.LightGreen;
                    }catch(Exception ex){probe.Content="● Ошибка: "+ex.Message;probe.Foreground=Brushes.Salmon;}finally{probe.IsEnabled=true;}};
                }
            }
            action(panel,"＋  Добавить подключение",()=>editor(-1));
            }
            label("Данные плагина",20);var files=new WrapPanel();panel.Children.Add(files);
            action(files,"Импорт JSON",import);
            action(files,"Экспорт JSON",delegate{var d=new SaveFileDialog{Filter="JSON|*.json",FileName="plugin-data.json"};if(d.ShowDialog(owner)==true)Save(d.FileName,v);});
            action(panel,"Удалить плагин",delegate{if(MessageBox.Show(owner,"Удалить плагин и его настройки?","Плагины",MessageBoxButton.YesNo)==MessageBoxResult.Yes){File.Delete(Path.Combine(Path.GetDirectoryName(path),"plugin","client.sessions.json"));File.Delete(Path.ChangeExtension(path,"connector-login"));File.Delete(module);File.Delete(path);details=false;refresh();}});
        };
        refresh();
    }
}
