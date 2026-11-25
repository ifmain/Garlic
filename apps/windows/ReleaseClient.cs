using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Net;
using System.Reflection;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Threading.Tasks;
using System.Web.Script.Serialization;
using System.Windows;

internal static class ReleaseClient
{
    const string Asset = "Garlic-Alpha-Windows-x64.exe";
    static string Root { get { return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "Garlic Alpha"); } }
    internal static bool Prepare()
    {
        string self = Assembly.GetExecutingAssembly().Location;
        if (!new WindowsPrincipal(WindowsIdentity.GetCurrent()).IsInRole(WindowsBuiltInRole.Administrator))
        {
            try { Process.Start(new ProcessStartInfo(self) { UseShellExecute = true, Verb = "runas", WorkingDirectory = Environment.SystemDirectory }); }
            catch (System.ComponentModel.Win32Exception e)
            {
                if (e.NativeErrorCode != 1223) throw;
                Environment.ExitCode = 1223;
                MessageBox.Show("Необходимо подтвердить выдачу прав администратора.", "Garlic Alpha", MessageBoxButton.OK, MessageBoxImage.Warning);
            }
            return false;
        }
        Directory.CreateDirectory(Root);
        Version version = Version.Parse(GarlicBuild.Version);
        string folder = Path.Combine(Root, version.ToString());
        string installed = Path.Combine(folder, "Garlic.exe");
        InstallBundle(Root, folder, self);
        if (Array.IndexOf(Environment.GetCommandLineArgs(), "--install-only") >= 0) return false;
        foreach (string dir in Directory.GetDirectories(Root))
        {
            Version v;
            if (Version.TryParse(Path.GetFileName(dir), out v) && v > version && (Previews || File.Exists(Path.Combine(dir,"stable.release"))) && File.Exists(Path.Combine(dir,"Garlic.exe")))
            { version = v; installed = Path.Combine(dir,"Garlic.exe"); }
        }
        if (string.Equals(self, installed, StringComparison.OrdinalIgnoreCase)) return true;
        Process.Start(new ProcessStartInfo(installed) { UseShellExecute = false, WorkingDirectory = Path.GetDirectoryName(installed) });
        return false;
    }

    private static void InstallBundle(string root, string folder, string self)
    {
        string installed = Path.Combine(folder, "Garlic.exe");
        if (!File.Exists(installed))
        {
            string stage = Directory.CreateDirectory(Path.Combine(root, ".install-" + Guid.NewGuid().ToString("N"))).FullName;
            try
            {
                using (Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("Garlic.Bundle"))
                using (var archive = new ZipArchive(stream, ZipArchiveMode.Read))
                foreach (ZipArchiveEntry entry in archive.Entries)
                {
                    string path = Path.GetFullPath(Path.Combine(stage, entry.FullName));
                    if (!path.StartsWith(stage + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase)) throw new InvalidDataException();
                    if (entry.Name.Length == 0) continue;
                    Directory.CreateDirectory(Path.GetDirectoryName(path));
                    using (Stream input = entry.Open())
                    using (Stream output = new FileStream(path, FileMode.CreateNew, FileAccess.Write)) input.CopyTo(output);
                }
                File.Copy(self, Path.Combine(stage, "Garlic.exe"));
                try { Directory.Move(stage, folder); } catch (IOException) { if (!File.Exists(installed)) throw; }
            }
            finally { if (Directory.Exists(stage)) Directory.Delete(stage, true); }
        }
    }

    internal static byte[] Fetch(string url, int limit)
    {
        var req = (HttpWebRequest)WebRequest.Create(url);
        req.UserAgent = "Garlic-Alpha/" + GarlicBuild.Version;
        req.Timeout = req.ReadWriteTimeout = 15000;
        using (var res = (HttpWebResponse)req.GetResponse())
        using (var stream = res.GetResponseStream())
        using (var result = new MemoryStream())
        {
            if (res.ResponseUri.Scheme != "https" || res.ContentLength > limit) throw new InvalidDataException();
            byte[] buf = new byte[32768]; int n;
            while ((n = stream.Read(buf,0,buf.Length)) > 0)
            {
                if (result.Length + n > limit) throw new InvalidDataException();
                result.Write(buf,0,n);
            }
            return result.ToArray();
        }
    }
    internal static bool Authentic(byte[] file, byte[] signature)
    {
        using (var reader = new StreamReader(Assembly.GetExecutingAssembly().GetManifestResourceStream("Garlic.ReleasePublicKey")))
        using (var rsa = new RSACryptoServiceProvider(new CspParameters(24) { Flags = CspProviderFlags.CreateEphemeralKey }))
        { rsa.FromXmlString(reader.ReadToEnd()); return rsa.VerifyData(file, "SHA256", signature); }
    }
    internal static bool SupportWarning(string notes, string installed, string offered)
    {
        if (String.IsNullOrWhiteSpace(notes)) return false;
        string first = notes.Trim().Split(new[] {'\n'}, 2)[0].Trim();
        var marker = System.Text.RegularExpressions.Regex.Match(first,
            @"^<!-- garlic-support-floor: ([0-9]{1,6}(?:\.[0-9]{1,6}){2,3}) -->$");
        if (!marker.Success) return false;
        Version floor = SupportVersion(marker.Groups[1].Value);
        return floor > SupportVersion(installed) && floor <= SupportVersion(offered);
    }
    static Version SupportVersion(string value)
    {
        Version version = Version.Parse(value);
        return new Version(version.Major,version.Minor,Math.Max(0,version.Build),Math.Max(0,version.Revision));
    }
    static string ChannelFile { get { return Path.ChangeExtension(Environment.GetEnvironmentVariable("GARLIC_CLIENT_CONFIG"),"include-prereleases"); } }
    internal static bool Previews { get { return File.Exists(ChannelFile); } }
    internal static void SetPreviews(bool enable)
    {
        if (enable) {
            Directory.CreateDirectory(Path.GetDirectoryName(ChannelFile));
            File.WriteAllText(ChannelFile,"");
        } else File.Delete(ChannelFile);
    }
    internal static Dictionary<string,object> SelectRelease(Dictionary<string,object>[] releases, bool previews)
    {
        Version installed = Version.Parse(GarlicBuild.Version);
        Dictionary<string,object> selected = null;
        string newest = ""; Version selectedVersion = installed;
        foreach (var release in releases) {
            string tag = release["tag_name"] as string;
            Version v;
            if (!object.Equals(release["draft"],false) || !(object.Equals(release["prerelease"],false) || previews)
                || tag == null || !tag.StartsWith("alpha-",StringComparison.Ordinal)
                || !Version.TryParse(tag.Substring(6),out v) || v.ToString()!=tag.Substring(6) || v<=installed) continue;
            string date=(string)release["published_at"];
            int order=string.CompareOrdinal(date,newest);
            if (order>0 || (order==0 && v>selectedVersion)) { newest=date; selectedVersion=v; selected=release; }
        }
        return selected;
    }
    static Dictionary<string,object> Latest()
    {
        bool previews=Previews;
        Dictionary<string,object> selected=null;
        for (int page=1; ; page++) {
            var releases = new JavaScriptSerializer().Deserialize<Dictionary<string,object>[]>(System.Text.Encoding.UTF8.GetString(
                Fetch("https://api.github.com/repos/ifmain/Garlic/releases?per_page=30&page="+page,4*1024*1024)));
            int count=releases.Length;
            if(selected!=null) { Array.Resize(ref releases,count+1); releases[count]=selected; }
            selected=SelectRelease(releases,previews);
            if (count<30) return selected;
        }
    }
    static void Install(string tag, bool preview)
    {
        string url = "https://github.com/ifmain/Garlic/releases/download/"+tag+"/"+Asset;
        byte[] signature = Fetch(url+".sig",1024);
        byte[] file = Fetch(url,64*1024*1024);
        if (!Authentic(file,signature)) throw new InvalidDataException("Подпись обновления не подтверждена.");
        string tmp = Path.Combine(Root,".update-"+Guid.NewGuid().ToString("N")+".exe");
        try {
            File.WriteAllBytes(tmp,file);
            if (FileVersionInfo.GetVersionInfo(tmp).ProductVersion != tag.Substring(6)) throw new InvalidDataException("Неверная версия обновления.");
            if(preview && !Previews) throw new OperationCanceledException("Источник обновлений изменился.");
            using (var p = Process.Start(new ProcessStartInfo(tmp,"--install-only") {UseShellExecute=false,WorkingDirectory=Root}))
            { if (!p.WaitForExit(60000) || p.ExitCode != 0 || !File.Exists(Path.Combine(Root,tag.Substring(6),"Garlic.exe"))) throw new IOException("Не удалось установить обновление."); }
            if(!preview) File.WriteAllText(Path.Combine(Root,tag.Substring(6),"stable.release"),"");
        } finally { if (File.Exists(tmp)) File.Delete(tmp); }
    }
    internal static string RestartPath;
    static bool Prompt(Window owner, string title, string message, string action, string cancel)
    {
        var dialog = new Window { Owner=owner, Title=title, Icon=owner.Icon,
            SizeToContent=SizeToContent.WidthAndHeight, ResizeMode=ResizeMode.NoResize,
            WindowStartupLocation=WindowStartupLocation.CenterOwner, ShowInTaskbar=false };
        var body = new System.Windows.Controls.StackPanel { Margin=new Thickness(24), Width=340 };
        if(owner.Icon!=null) body.Children.Add(new System.Windows.Controls.Image {Source=owner.Icon, Width=40, Height=40});
        body.Children.Add(new System.Windows.Controls.TextBlock { Text=message, TextWrapping=TextWrapping.Wrap,
            Margin=new Thickness(0,12,0,16), FontSize=16 });
        var buttons=new System.Windows.Controls.StackPanel {Orientation=System.Windows.Controls.Orientation.Horizontal,
            HorizontalAlignment=HorizontalAlignment.Right};
        foreach(string label in new[]{cancel,action}) {
            if(label==null) continue;
            var button=new System.Windows.Controls.Button { Content=label, Padding=new Thickness(12,6,12,6),Margin=new Thickness(4), IsDefault=label==action };
            bool confirm=label==action;
            button.Click+=delegate {dialog.DialogResult=confirm;};
            buttons.Children.Add(button);
        }
        body.Children.Add(buttons); dialog.Content=body;
        return dialog.ShowDialog()==true;
    }
    static void Installed(Window owner,string version,bool required)
    {
        if(Prompt(owner,"Garlic Alpha","Версия "+version+" установлена. Перезапустить Garlic сейчас?",
                "Перезапустить Garlic",required ? null : "ОК")) {
            RestartPath=Path.Combine(Root,version,"Garlic.exe");
            Application.Current.Shutdown();
        } else if(required) Application.Current.Shutdown();
    }
    static bool checking, manualRequested;
    internal static async void Check(Window window, bool manual = false)
    {
        manualRequested |= manual;
        if (checking) return;
        checking = true;
        bool accepted = false;
        try
        {
            ServicePointManager.SecurityProtocol |= SecurityProtocolType.Tls12;
            var release = await Task.Run(() => Latest());
            bool preview = release!=null && object.Equals(release["prerelease"],true);
            string tag = release==null || (preview && !Previews) ? null : (string)release["tag_name"];
            if (!window.IsVisible) return;
            if (tag == null) {
                if(manualRequested) MessageBox.Show(window,"Установлена последняя версия: "+GarlicBuild.Version,"Garlic Alpha");
                return;
            }
            object raw;
            string notes=release.TryGetValue("body",out raw) ? raw as string : "";
            string version=tag.Substring(6);
            bool important=SupportWarning(notes,GarlicBuild.Version,version);
            bool required=notes!=null && notes.TrimStart().StartsWith("<!-- garlic-required-floor:",StringComparison.Ordinal)
                && SupportWarning(notes.Replace("garlic-required-floor:","garlic-support-floor:"),GarlicBuild.Version,version);
            if(File.Exists(Path.Combine(Root,version,"Garlic.exe"))) {
                if(!preview) File.WriteAllText(Path.Combine(Root,version,"stable.release"),"");
                Installed(window,version,required);
                return;
            }
            accepted=Prompt(window,required ? "Необходимо обновить Garlic" : "Обновление Garlic",
                (required ? "Для продолжения работы требуется обновление. " : important ? "Поддержка вашей версии скоро закончится. " : "")+
                "Доступен "+(preview ? "пререлиз " : "релиз ")+version+".","Обновить",required ? null : "Позже");
            if (!accepted || (preview && !Previews)) {
                if(required && (!preview || Previews)) Application.Current.Shutdown();
                return;
            }
            if(required) window.IsEnabled=false;
            try { await Task.Run(() => Install(tag,preview)); }
            catch { if(required) Application.Current.Shutdown(); throw; }
            finally { window.IsEnabled=true; }
            if(!preview || Previews) Installed(window,version,required);
        }
        catch(Exception e) { GarlicApp.LogUi("update_error",e.GetType().Name); if ((accepted || manualRequested) && window.IsVisible) MessageBox.Show(window,e.Message,"Обновление Garlic"); }
        finally { checking = false; manualRequested = false; }
    }
}
