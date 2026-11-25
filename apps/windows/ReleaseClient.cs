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
            if (Version.TryParse(Path.GetFileName(dir), out v) && v > version && File.Exists(Path.Combine(dir,"Garlic.exe")))
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
    static string Latest()
    {
        var releases = new JavaScriptSerializer().Deserialize<Dictionary<string,object>[]>(System.Text.Encoding.UTF8.GetString(
            Fetch("https://api.github.com/repos/ifmain/Garlic/releases?per_page=30", 1024*1024)));
        Version newest = Version.Parse(GarlicBuild.Version); string selected = null;
        foreach (var release in releases)
        {
            string tag = release["tag_name"] as string; Version v;
            if (object.Equals(release["draft"],false) && object.Equals(release["prerelease"],false) &&
                tag != null && tag.StartsWith("alpha-",StringComparison.Ordinal) && Version.TryParse(tag.Substring(6),out v) &&
                v.ToString() == tag.Substring(6) && v > newest) { newest = v; selected = tag; }
        }
        return selected;
    }
    static void Install(string tag)
    {
        string url = "https://github.com/ifmain/Garlic/releases/download/"+tag+"/"+Asset;
        byte[] signature = Fetch(url+".sig",1024);
        byte[] file = Fetch(url,64*1024*1024);
        if (!Authentic(file,signature)) throw new InvalidDataException("Подпись обновления не подтверждена.");
        string tmp = Path.Combine(Root,".update-"+Guid.NewGuid().ToString("N")+".exe");
        try {
            File.WriteAllBytes(tmp,file);
            if (FileVersionInfo.GetVersionInfo(tmp).ProductVersion != tag.Substring(6)) throw new InvalidDataException("Неверная версия обновления.");
            using (var p = Process.Start(new ProcessStartInfo(tmp,"--install-only") {UseShellExecute=false,WorkingDirectory=Root}))
            { if (!p.WaitForExit(60000) || p.ExitCode != 0 || !File.Exists(Path.Combine(Root,tag.Substring(6),"Garlic.exe"))) throw new IOException("Не удалось установить обновление."); }
        } finally { if (File.Exists(tmp)) File.Delete(tmp); }
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
            string tag = await Task.Run((Func<string>)Latest);
            if (!window.IsVisible) return;
            if (tag == null || File.Exists(Path.Combine(Root,tag.Substring(6),"Garlic.exe"))) {
                if (manualRequested) MessageBox.Show(window,tag == null ? "Установлена последняя версия: "+GarlicBuild.Version : "Обновление уже установлено. Перезапустите Garlic.","Обновление Garlic");
                return;
            }
            accepted = MessageBox.Show(window,"Скачать и установить Garlic Alpha " + tag.Substring(6) + "? Обновление откроется при следующем запуске.",
                "Обновление Garlic",MessageBoxButton.YesNo,MessageBoxImage.Question) == MessageBoxResult.Yes;
            if (!accepted) return;
            await Task.Run(() => Install(tag));
            if(window.IsVisible) MessageBox.Show(window,"Обновление установлено. При следующем запуске откроется новая версия.","Garlic Alpha");
        }
        catch(Exception e) { GarlicApp.LogUi("update_error",e.GetType().Name); if ((accepted || manualRequested) && window.IsVisible) MessageBox.Show(window,e.Message,"Обновление Garlic"); }
        finally { checking = false; manualRequested = false; }
    }
}
