using System;
using System.ComponentModel;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Windows;
using Forms = System.Windows.Forms;

internal sealed class TrayWindow : IDisposable
{
    readonly Application app = Application.Current;
    readonly Window window;
    readonly Forms.NotifyIcon icon;
    readonly Icon image;
    readonly Forms.ContextMenuStrip menu = new Forms.ContextMenuStrip();
    readonly Forms.ToolStripMenuItem hideOnClose;
    readonly RegisteredWaitHandle activation;
    readonly string disableFile;
    bool exiting;
    bool disposed;
    WindowState previousState = WindowState.Normal;

    internal TrayWindow(Window window, string config, EventWaitHandle showSignal)
    {
        this.window = window;
        disableFile = Path.ChangeExtension(config, "close-without-tray");
        menu.Items.Add("Открыть Garlic", null, delegate { Restore(); });
        hideOnClose = new Forms.ToolStripMenuItem("Сворачивать в трей при закрытии") {
            Checked = !File.Exists(disableFile), CheckOnClick = true
        };
        hideOnClose.Click += delegate {
            try {
                if (hideOnClose.Checked) File.Delete(disableFile);
                else {
                    Directory.CreateDirectory(Path.GetDirectoryName(disableFile));
                    File.WriteAllText(disableFile, "");
                }
            } catch (Exception ex) {
                hideOnClose.Checked = !File.Exists(disableFile);
                MessageBox.Show(ex.Message, "Garlic");
            }
        };
        menu.Items.Add(hideOnClose);
        menu.Items.Add(new Forms.ToolStripSeparator());
        menu.Items.Add("Выход из Garlic", null, delegate { exiting = true; window.Close(); });
        image = Icon.ExtractAssociatedIcon(Assembly.GetExecutingAssembly().Location);
        icon = new Forms.NotifyIcon { Icon = image, Text = "Garlic Alpha", ContextMenuStrip = menu, Visible = true };
        icon.MouseClick += delegate(object sender, Forms.MouseEventArgs e) {
            if (e.Button == Forms.MouseButtons.Left) Restore();
        };
        window.Closing += Closing;
        app.SessionEnding += SessionEnding;
        activation = ThreadPool.RegisterWaitForSingleObject(showSignal, delegate {
            if (!window.Dispatcher.HasShutdownStarted)
                window.Dispatcher.BeginInvoke(new Action(Restore));
        }, null, Timeout.Infinite, false);
    }

    void Closing(object sender, CancelEventArgs e)
    {
        if (exiting || !hideOnClose.Checked) return;
        e.Cancel = true;
        if (window.WindowState != WindowState.Minimized) previousState = window.WindowState;
        window.Hide();
    }

    void SessionEnding(object sender, SessionEndingCancelEventArgs e) { exiting = true; }

    void Restore()
    {
        if (disposed || exiting) return;
        if (!window.IsVisible) window.Show();
        if (window.WindowState == WindowState.Minimized) window.WindowState = previousState;
        window.Activate();
    }

    public void Dispose()
    {
        if (disposed) return;
        disposed = true;
        activation.Unregister(null);
        window.Closing -= Closing;
        app.SessionEnding -= SessionEnding;
        icon.Visible = false;
        icon.Dispose();
        menu.Dispose();
        image.Dispose();
    }
}
