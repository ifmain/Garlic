using System;
using System.Collections;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using System.Web.Script.Serialization;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Markup;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;

[assembly: AssemblyInformationalVersion(GarlicBuild.Version)]
[assembly: RuntimeCompatibility(WrapNonExceptionThrows = true)]
[assembly: CompilationRelaxations(8)]
[assembly: AssemblyVersion("0.0.0.0")]
public sealed class GarlicApp
{
	private sealed class ServerRow
	{
		public string Id;

		public uint Streams;

		public uint Workers;

		public string Name { get; set; }

		public string Status { get; set; }

		public string Address { get; set; }
	}

	private sealed class UserRow
	{
		public Dictionary<string, object> Data;

		public string Name { get; set; }

		public string Detail { get; set; }
	}

	private sealed class LegacyEntry
	{
		public string Email;

		public string Label;

		public override string ToString()
		{
			return Label;
		}
	}

	private readonly Window window;

	private readonly string root;

	private readonly string bundled;

	private string binary;

	private FileStream coreLock;

	private readonly Dictionary<string, object> controls = new Dictionary<string, object>();

	private readonly JavaScriptSerializer json = new JavaScriptSerializer();

	private readonly DispatcherTimer timer = new DispatcherTimer();

	private bool admin;

	private bool connected;

	private bool pending;

	private bool polling;

	private bool changingConnection;

	private bool signingOut;

	private int routeMode = 0;

	private int tunnelMode = 0;

	private int activeTunnelMode = -1;

	private Process vpn;

	private object savedAccount;

	private int accountGeneration;

	private long lastStatsTick;

	private long lastRx;

	private long lastTx;

	private readonly string[] tunnelNames = new string[2] { "VPN (вся система)", "Прокси SOCKS5 (127.0.0.1:10888)" };

	private readonly string[] tunnelHints = new string[2] { "Маршрутизирует весь трафик устройства (требуются права администратора).", "Локальный SOCKS5 / HTTP прокси на 127.0.0.1:10888 для браузера без прав администратора." };

	private DateTime expiryDate = DateTime.Today.AddMonths(1);

	private readonly CultureInfo culture = CultureInfo.GetCultureInfo("ru-RU");

	private static string UiDataDirectory()
	{
		string environmentVariable = Environment.GetEnvironmentVariable("GARLIC_CLIENT_CONFIG");
		return string.IsNullOrEmpty(environmentVariable) ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Garlic") : Path.GetDirectoryName(Path.GetFullPath(environmentVariable));
	}

	private string UiModePath()
	{
		return Path.Combine(UiDataDirectory(), "client.ui-mode.txt");
	}

	internal static void LogUi(string eventName, string detail)
	{
		try
		{
			DateTime utc = DateTime.UtcNow;
			string dir = Path.Combine(UiDataDirectory(), "logs");
			Directory.CreateDirectory(dir);
			string path = Path.Combine(dir, utc.ToString("yyyy-MM-dd", System.Globalization.CultureInfo.InvariantCulture) + ".log");
			if (!File.Exists(path) || new FileInfo(path).Length < 8 * 1024 * 1024)
				File.AppendAllText(path, utc.ToString("o") + " " + eventName + " " + detail + Environment.NewLine);
		}
		catch (IOException) {}
		catch (UnauthorizedAccessException) {}
	}

	private void ApplyTunnelMode(int selected, bool save = true)
	{
		tunnelMode = ((selected == 1) ? 1 : 0);
		C<Button>("TunnelMode").Content = tunnelNames[tunnelMode] + "  ⌄";
		C<TextBlock>("TunnelHint").Text = tunnelHints[tunnelMode];
		if (save)
		{
			try
			{
				string path = UiModePath();
				Directory.CreateDirectory(Path.GetDirectoryName(path));
				File.WriteAllText(path, (tunnelMode == 1) ? "proxy" : "vpn");
			}
			catch
			{
			}
		}
	}

	private T C<T>(string name)
	{
		return (T)controls[name];
	}

	private static Dictionary<string, object> Map(object value)
	{
		return (value as Dictionary<string, object>) ?? new Dictionary<string, object>();
	}

	private static object Get(object value, string name)
	{
		object value2;
		return Map(value).TryGetValue(name, out value2) ? value2 : null;
	}

	private static string Text(object value)
	{
		return Convert.ToString(value);
	}

	private void Say(string value)
	{
		C<TextBlock>("Message").Text = value;
	}

	private void Click(string name, Func<Task> action)
	{
		C<Button>(name).Click += async (object sender, RoutedEventArgs e) =>
		{
			try
			{
				await action();
			}
			catch (Exception ex)
			{
				LogUi("ui_error", name + " " + ex.GetType().Name + " hresult=" + ex.HResult);
				Say(ex.Message);
			}
		};
	}

	private void DateLabel()
	{
		C<Button>("Days").Content = expiryDate.ToString("d MMMM yyyy", culture) + "  ⌄";
	}

	private void Calendar()
	{
		Popup popup = new Popup
		{
			PlacementTarget = C<Button>("Days"),
			Placement = PlacementMode.Bottom,
			StaysOpen = false,
			AllowsTransparency = true
		};
		StackPanel body = new StackPanel
		{
			Width = 252.0,
			Margin = new Thickness(12.0)
		};
		body.Resources = window.Resources;
		popup.Child = new Border
		{
			Background = new SolidColorBrush(Color.FromRgb(40, 40, 40)),
			CornerRadius = new CornerRadius(12.0),
			BorderThickness = new Thickness(1.0),
			BorderBrush = Brushes.DimGray,
			Child = body
		};
		DateTime month = new DateTime(expiryDate.Year, expiryDate.Month, 1);
		Action render = null;
		render = () =>
		{
			body.Children.Clear();
			DockPanel dockPanel = new DockPanel
			{
				Margin = new Thickness(0.0, 0.0, 0.0, 16.0)
			};
			Button button = new Button
			{
				Content = "‹",
				Padding = new Thickness(12.0, 5.0, 12.0, 5.0)
			};
			Button button2 = new Button
			{
				Content = "›",
				Padding = button.Padding
			};
			DockPanel.SetDock(button, Dock.Left);
			DockPanel.SetDock(button2, Dock.Right);
			dockPanel.Children.Add(button);
			dockPanel.Children.Add(button2);
			dockPanel.Children.Add(new TextBlock
			{
				Text = month.ToString("MMMM yyyy", culture),
				Foreground = Brushes.White,
				VerticalAlignment = VerticalAlignment.Center,
				TextAlignment = TextAlignment.Center
			});
			body.Children.Add(dockPanel);
			button.Click += (object s, RoutedEventArgs e) =>
			{
				month = month.AddMonths(-1);
				render();
			};
			button2.Click += (object s, RoutedEventArgs e) =>
			{
				month = month.AddMonths(1);
				render();
			};
			UniformGrid uniformGrid = new UniformGrid
			{
				Columns = 7
			};
			string[] array = new string[7] { "пн", "вт", "ср", "чт", "пт", "сб", "вс" };
			foreach (string text in array)
			{
				uniformGrid.Children.Add(new TextBlock
				{
					Text = text,
					Foreground = Brushes.Gray,
					TextAlignment = TextAlignment.Center,
					Margin = new Thickness(0.0, 0.0, 0.0, 10.0)
				});
			}
			int num2 = (int)(month.DayOfWeek + 6) % 7;
			for (int num3 = 0; num3 < num2; num3++)
			{
				uniformGrid.Children.Add(new TextBlock());
			}
			for (int num4 = 1; num4 <= DateTime.DaysInMonth(month.Year, month.Month); num4++)
			{
				DateTime day = month.AddDays(num4 - 1);
				bool flag = day.Date == expiryDate.Date;
				Button button3 = new Button
				{
					Content = num4.ToString(),
					Height = 36.0,
					Margin = new Thickness(1.0),
					Padding = new Thickness(0.0),
					Background = (flag ? Brushes.White : Brushes.Transparent),
					Foreground = (flag ? Brushes.Black : Brushes.White)
				};
				button3.Click += (object s, RoutedEventArgs e) =>
				{
					expiryDate = day;
					DateLabel();
					popup.IsOpen = false;
				};
				uniformGrid.Children.Add(button3);
			}
			body.Children.Add(uniformGrid);
			body.Children.Add(new TextBlock
			{
				Text = "Выбранная дата включительно",
				Foreground = Brushes.Gray,
				FontSize = 12.0,
				Margin = new Thickness(0.0, 14.0, 0.0, 0.0)
			});
		};
		render();
		popup.IsOpen = true;
	}

	private static Process StartNative(ProcessStartInfo start)
	{
		try
		{
			return Process.Start(start);
		}
		catch (Win32Exception ex)
		{
			if (ex.NativeErrorCode == 4551)
			{
				throw new Exception("Контроль приложений Windows заблокировал неподписанный сетевой модуль Garlic. Для этой системы нужна сборка с доверенной подписью издателя. Сетевые настройки не изменены.", ex);
			}
			throw;
		}
	}

	private async Task<Dictionary<string, object>> Cli(string[] arguments, string input = null, int timeoutMs = 25000, string executable = null)
	{
		foreach (string text in arguments)
		{
			if (text.IndexOfAny(new char[3] { '"', '\r', '\n' }) >= 0)
			{
				throw new Exception("Некорректный аргумент");
			}
		}
		if (true)
		{
		}
		using (Process process = StartNative(new ProcessStartInfo(executable ?? binary, string.Join(" ", arguments.Select((string a) => "\"" + a + "\"")))
		{
			UseShellExecute = false,
			CreateNoWindow = true,
			RedirectStandardInput = true,
			RedirectStandardOutput = true,
			RedirectStandardError = true,
			StandardOutputEncoding = Encoding.UTF8,
			StandardErrorEncoding = Encoding.UTF8
		}))
		{
			if (input != null)
			{
				byte[] utf8 = Encoding.UTF8.GetBytes(input + "\n");
				await process.StandardInput.BaseStream.WriteAsync(utf8, 0, utf8.Length);
			}
			process.StandardInput.Close();
			Task<string> output = process.StandardOutput.ReadToEndAsync();
			Task<string> errors = process.StandardError.ReadToEndAsync();
			Task<string[]> completed = Task.WhenAll<string>(output, errors);
			if (await Task.WhenAny(completed, Task.Delay(timeoutMs)) != completed)
			{
				try
				{
					process.Kill();
				}
				catch
				{
				}
				throw new Exception("Операция не завершилась вовремя. Аккаунт сохранён, попробуйте снова.");
			}
			await completed;
			process.WaitForExit();
			if (process.ExitCode != 0)
			{
				throw new Exception((errors.Result.Trim().Length > 0) ? errors.Result.Trim() : "Сетевой модуль не выполнил запрос");
			}
			try
			{
				return json.Deserialize<Dictionary<string, object>>(output.Result) ?? new Dictionary<string, object>();
			}
			catch
			{
				return new Dictionary<string, object>();
			}
		}
	}

	private async Task SelectCore()
	{
		Dictionary<string, object> selected = await Cli(new string[2] { "core-update", "select" }, null, 30000, bundled);
		string digest = Text(Get(selected, "sha256"));
		if (digest.Length == 0)
		{
			return;
		}
		if (!Regex.IsMatch(digest, "\\A[0-9a-f]{64}\\z"))
		{
			throw new IOException("Некорректная подпись модуля");
		}
		string target = Path.Combine(Path.GetDirectoryName(bundled), "garlic-core-" + digest + ".exe");
		if (!File.Exists(target))
		{
			string text = target + "." + Guid.NewGuid().ToString("N") + ".tmp";
			try
			{
				File.Copy(Text(Get(selected, "path")), text);
				File.Move(text, target);
			}
			finally
			{
				if (File.Exists(text))
				{
					File.Delete(text);
				}
			}
		}
		FileStream held = File.Open(target, FileMode.Open, FileAccess.Read, FileShare.Read);
		try
		{
			using (SHA256 sHA = SHA256.Create())
			{
				string text2 = BitConverter.ToString(sHA.ComputeHash(held)).Replace("-", "").ToLowerInvariant();
				if (text2 != digest)
				{
					throw new IOException("Контрольная сумма модуля не совпадает");
				}
			}
			Dictionary<string, object> info = await Cli(new string[1] { "version" }, null, 30000, target);
			if (Text(Get(info, "core_abi")) != "1" || Text(Get(info, "version")) != Text(Get(selected, "version")))
			{
				throw new IOException("Модуль несовместим с приложением");
			}
			if (object.Equals(Get(selected, "trial"), true))
			{
				await Cli(new string[3]
				{
					"core-update",
					"confirm",
					Text(Get(selected, "sequence"))
				}, null, 10000, bundled);
			}
			binary = target;
			coreLock = held;
			held = null;
		}
		finally
		{
			if (held != null)
			{
				held.Dispose();
			}
		}
	}

	private async Task StageCore()
	{
		try
		{
			await Cli(new string[2] { "core-update", "stage" }, null, 240000, bundled);
		}
		catch (Exception)
		{
		}
	}

	private void Home(object account, bool verified = true)
	{
		savedAccount = account;
		SelectSection("HomeButton");
		admin = verified && Text(Get(Get(account, "principal"), "role")) == "admin";
		C<FrameworkElement>("Auth").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Admin").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Servers").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Home").Visibility = Visibility.Visible;
		C<Button>("UsersButton").Visibility = ((!admin) ? Visibility.Collapsed : Visibility.Visible);
		C<Button>("ServersButton").Visibility = ((!admin) ? Visibility.Collapsed : Visibility.Visible);
		C<Button>("Takeover").Visibility = (admin ? Visibility.Collapsed : Visibility.Visible);
		object value = Get(account, "subscription");
		string text = "Подписка не проверена";
		if (admin)
		{
			text = "Администратор · Безлимит";
		}
		else if (Text(Get(value, "kind")) == "unlimited")
		{
			text = "Подписка · Безлимит";
		}
		else if (Get(value, "expires_at") != null)
		{
			double num = Convert.ToDouble(Get(value, "expires_at")) - (DateTime.UtcNow - new DateTime(1970, 1, 1)).TotalSeconds;
			text = ((num > 0.0) ? ("Активна до " + new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc).AddSeconds(Convert.ToDouble(Get(value, "expires_at"))).ToLocalTime().ToString("d MMMM yyyy", culture) + " · осталось дней: " + Math.Ceiling(num / 86400.0)) : "Подписка истекла");
		}
		C<TextBlock>("Subscription").Text = text;
	}

	private async Task StageInterface()
	{
		try
		{
			await Cli(new string[1] { "stage" }, null, 240000, Path.Combine(Path.GetDirectoryName(bundled), "garlic-ui-update.exe"));
		}
		catch (Exception)
		{
		}
	}

	private async Task Login()
	{
		string address = C<TextBox>("Server").Text.Trim();
		string key = C<PasswordBox>("Key").Password;
		if (address.Length == 0 || key.Length == 0)
		{
			Say("Введите сервер и ключ");
			return;
		}
		if (!address.Contains("://"))
		{
			address = "https://" + address + (address.Contains(":") ? "" : ":8443");
		}
		C<Button>("Login").IsEnabled = false;
		Say("Проверяем ключ и получаем сеть…");
		try
		{
			Dictionary<string, object> result = await Cli(new string[3] { "connect", address, "--key-stdin" }, key);
			C<PasswordBox>("Key").Clear();
			Home(Get(result, "account"));
#if !GARLIC_ALPHA
			StageInterface();
#endif
			Say("");
		}
		finally
		{
			C<Button>("Login").IsEnabled = true;
		}
	}

	private async Task StartVpn()
	{
		if (!signingOut && savedAccount != null)
		{
			int request = accountGeneration;
			string mode = (new string[3] { "auto", "direct", "ru" })[routeMode];
			await Cli(new string[4] { "config", "set", "mode", mode });
			if (request == accountGeneration)
			{
				ProcessStartInfo start = ((tunnelMode != 1) ? new ProcessStartInfo(binary, "vpn")
				{
					WorkingDirectory = Path.GetDirectoryName(binary),
#if GARLIC_BUNDLE
                    UseShellExecute = false,
                    CreateNoWindow = true,
#else
                    UseShellExecute = true,
                    Verb = "runas",
#endif
					WindowStyle = ProcessWindowStyle.Hidden
				} : new ProcessStartInfo(binary, "proxy --bind 127.0.0.1:10888")
				{
					WorkingDirectory = Path.GetDirectoryName(binary),
					UseShellExecute = false,
					CreateNoWindow = true
				});
				vpn = StartNative(start);
				activeTunnelMode = tunnelMode;
				pending = true;
				Say((tunnelMode == 1) ? "Запускаем SOCKS5 прокси на 127.0.0.1:10888…" : "Подключаемся…");
			}
		}
	}

	private async Task StopVpn()
	{
		await Cli(new string[1] { "stop" });
		Process running = vpn;
		if (running != null && !running.HasExited)
		{
			try
			{
				if (!(await Task.Run(() => running.WaitForExit(12000))))
				{
					running.Kill();
				}
			}
			catch
			{
			}
		}
		pending = false;
		connected = false;
		activeTunnelMode = -1;
		lastStatsTick = 0L;
		C<TextBlock>("Statistics").Text = "";
		Say("Подключение остановлено");
	}

	private async Task Poll()
	{
		if (polling)
		{
			return;
		}
		polling = true;
		int mode = ((activeTunnelMode < 0) ? tunnelMode : activeTunnelMode);
		try
		{
			Dictionary<string, object> status = await Cli(new string[2]
			{
				"proxy-status",
				(mode == 1) ? "127.0.0.1:10888" : "127.0.0.1:1080"
			});
			string clockWarning = Text(Get(status, "clock_warning"));
			if (object.Equals(Get(status, "vpn_initializing"), true) || object.Equals(Get(status, "network_ready"), false))
			{
				connected = false;
				pending = true;
				C<TextBlock>("RouteInfo").Text = clockWarning;
				C<TextBlock>("Connection").Text = "Настраиваем VPN…";
				C<Button>("Power").Content = "Отключить";
				return;
			}
			long tick = Stopwatch.GetTimestamp();
			long rx = Convert.ToInt64(Get(status, "rx_bytes") ?? ((object)0));
			long tx = Convert.ToInt64(Get(status, "tx_bytes") ?? ((object)0));
			double elapsed = ((lastStatsTick == 0) ? 0.0 : ((double)(tick - lastStatsTick) / (double)Stopwatch.Frequency));
			if (elapsed > 0.0)
			{
				C<TextBlock>("Statistics").Text = string.Format(culture, "↓ {0:F1} КБ/с · ↑ {1:F1} КБ/с · отклик {2} мс", (double)Math.Max(0L, rx - lastRx) / elapsed / 1024.0, (double)Math.Max(0L, tx - lastTx) / elapsed / 1024.0, Get(status, "ping_ms") ?? ((object)0));
			}
			lastStatsTick = tick;
			lastRx = rx;
			lastTx = tx;
			object route = Get(status, "last_route");
			string bind = Text(Get(status, "proxy_bind"));
			string prefix;
			if (bind.EndsWith(":10888"))
			{
				prefix = "SOCKS5 127.0.0.1:10888";
			}
			else
			{
				prefix = ((bind.Length > 0) ? "VPN" : "");
			}
			string routeText = ((route == null) ? "" : (((Text(Get(route, "via")) == "direct") ? "Напрямую" : "Через Россию") + " · " + Text(Get(route, "origin"))));
			TextBlock textBlock = C<TextBlock>("RouteInfo");
			string text;
			if (prefix.Length > 0 && routeText.Length > 0)
			{
				text = prefix + " · " + routeText;
			}
			else
			{
				text = ((routeText.Length > 0) ? routeText : prefix);
			}
			textBlock.Text = text;
			object transport = Get(status, "transport");
			string protocol = ((transport != null && object.Equals(Get(transport, "confirmed"), true)) ? Text(Get(transport, "protocol")) : "определяется");
			TextBlock textBlock2 = C<TextBlock>("RouteInfo");
			textBlock2.Text = textBlock2.Text + "\nСогласовано: " + protocol + ((Get(transport, "fallback_reason") != null) ? " · совместимость" : "");
			if (Get(transport, "peer_version") != null)
			{
				TextBlock textBlock3 = C<TextBlock>("RouteInfo");
				textBlock3.Text = textBlock3.Text + " · сервер " + Text(Get(transport, "peer_version"));
			}
			if (route != null && Text(Get(route, "via")) != "direct")
			{
				string text2 = Text(Get(transport, "relay_protocol"));
				TextBlock textBlock4 = C<TextBlock>("RouteInfo");
				TextBlock textBlock5 = textBlock4;
				string text3 = textBlock4.Text;
				string text4;
				if (text2 == "relay-shaped-v2")
				{
					text4 = "Relay v2";
				}
				else
				{
					text4 = ((text2 == "relay-tls-v1") ? "Relay v1 · совместимость" : "версия не сообщена");
				}
				textBlock5.Text = text3 + "\nМежду нодами: " + text4;
			}
			object modes = Get(status, "transport_modes");
			List<string> liveModes = new List<string>();
			Dictionary<string, object> counts = modes as Dictionary<string, object>;
			if (counts != null)
			{
				foreach (KeyValuePair<string, object> item in counts.OrderBy((KeyValuePair<string, object> p) => p.Key, StringComparer.Ordinal))
				{
					if (Convert.ToInt64(item.Value) > 0)
					{
						liveModes.Add(item.Key);
					}
				}
			}
			TextBlock textBlock6 = C<TextBlock>("RouteInfo");
			TextBlock textBlock7 = textBlock6;
			string text5 = textBlock6.Text;
			string text6;
			if (liveModes.Count > 0)
			{
				text6 = string.Join(" + ", liveModes.ToArray());
			}
			else if (object.Equals(Get(status, "network_ready"), true))
			{
				text6 = ((counts == null) ? "нет данных" : "сейчас нет активных каналов");
			}
			else
			{
				text6 = "ожидание соединения";
			}
			textBlock7.Text = text5 + "\nКаналы: " + text6;
			if (!string.IsNullOrEmpty(clockWarning)) textBlock7.Text += "\n" + clockWarning;
			if (mode == 0 && !object.Equals(Get(status, "tunnel"), true))
			{
				throw new InvalidOperationException("Device tunnel is not running");
			}
			connected = true;
			pending = false;
			C<TextBlock>("Connection").Text = "Подключено";
			C<Button>("Power").Content = "Отключить";
		}
		catch
		{
			connected = false;
			lastStatsTick = 0L;
			C<TextBlock>("Statistics").Text = "";
			C<TextBlock>("RouteInfo").Text = "";
			if (pending && vpn != null && !vpn.HasExited)
			{
				C<TextBlock>("Connection").Text = "Подключение…";
				return;
			}
			pending = false;
			C<TextBlock>("Connection").Text = "Не подключено";
			C<Button>("Power").Content = "Подключить";
			try
			{
				string path = Path.Combine(UiDataDirectory(), "client.vpn-error.json");
				if (mode == 0 && File.Exists(path))
				{
					Say(Text(Get(json.DeserializeObject(File.ReadAllText(path)), "error")));
				}
			}
			catch
			{
			}
		}
		finally
		{
			polling = false;
		}
	}

	private void SelectSection(string selected)
	{
		string[] array = new string[3] { "HomeButton", "UsersButton", "ServersButton" };
		foreach (string text in array)
		{
			C<Button>(text).Background = ((text == selected) ? new SolidColorBrush(Color.FromRgb(46, 46, 46)) : Brushes.Transparent);
		}
	}

	private string UserExpiry(object user)
	{
		if (Text(Get(user, "role")) == "admin")
		{
			return "Администратор · Безлимит";
		}
		object value = Get(user, "subscription");
		if (Text(Get(value, "kind")) == "unlimited")
		{
			return "Безлимит";
		}
		object obj = Get(value, "expires_at");
		return (obj == null) ? "Срок не указан" : ("До " + new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc).AddSeconds(Convert.ToDouble(obj)).ToLocalTime().ToString("d MMMM yyyy", culture));
	}

	private async Task Users()
	{
		int request = accountGeneration;
		if (!admin)
		{
			return;
		}
		SelectSection("UsersButton");
		Say("Загрузка пользователей…");
		C<FrameworkElement>("Servers").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Home").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Admin").Visibility = Visibility.Visible;
		Dictionary<string, object> result = await Cli(new string[2] { "admin", "list" });
		if (request != accountGeneration || !admin)
		{
			return;
		}
		ListBox list = C<ListBox>("Users");
		list.Items.Clear();
		IEnumerable values = Get(result, "users") as IEnumerable;
		if (values != null)
		{
			foreach (object item in values)
			{
				Dictionary<string, object> dictionary = Map(item);
				dictionary["display"] = Text(Get(item, "username")) + " · " + Text(Get(item, "role")) + " · " + Text(Get(item, "subscription_display")) + (object.Equals(Get(item, "enabled"), false) ? " · отключён" : "");
				string text = Text(Get(item, "dns_id"));
				string text2;
				if (Text(Get(item, "status")) == "expired")
				{
					text2 = "закончилась";
				}
				else
				{
					text2 = ((Text(Get(item, "status")) == "disabled") ? "отключён" : "активна");
				}
				string value = Text(Get(item, "doh_url"));
				dictionary["doh_url"] = value;
				list.Items.Add(new UserRow
				{
					Data = dictionary,
					Name = Text(Get(item, "username")),
					Detail = UserExpiry(item) + " · " + text2 + "\n" + Text(Get(item, "client_usage_display"))
				});
			}
		}
		Say("");
	}

	private async Task Servers()
	{
		int request = accountGeneration;
		if (!admin)
		{
			return;
		}
		SelectSection("ServersButton");
		Say("Получаем состояние сети…");
		C<FrameworkElement>("Home").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Admin").Visibility = Visibility.Collapsed;
		C<FrameworkElement>("Servers").Visibility = Visibility.Visible;
		Dictionary<string, object> result = await Cli(new string[2] { "admin", "network" });
		if (request != accountGeneration || !admin)
		{
			return;
		}
		object network = Get(result, "network");
		ListBox list = C<ListBox>("Nodes");
		list.Items.Clear();
		C<Button>("SaveServer").IsEnabled = false;
		IEnumerable members = Get(network, "nodes") as IEnumerable;
		if (members != null)
		{
			foreach (object item in members)
			{
				string text = Text(Get(item, "node_id"));
				object value = Get(Get(network, "observations"), text);
				object value2 = Get(Get(network, "policies"), text);
				double num = ((Get(value, "last_seen") == null) ? double.MaxValue : ((DateTime.UtcNow - new DateTime(1970, 1, 1)).TotalSeconds - Convert.ToDouble(Get(value, "last_seen"))));
				string text2 = Text(Get(item, "label")) + " · " + ((Text(Get(item, "role")) == "ru") ? "РФ" : "Выход") + " · " + ((num < 30.0) ? "В сети" : "Нет свежих сведений") + "\n" + Text(Get(item, "url")) + "\nПотоков: " + Text(Get(value, "active_streams")) + " · таймаутов: " + Text(Get(value, "timeout_count"));
				list.Items.Add(new ServerRow
				{
					Id = text,
					Streams = ((Get(value2, "stream_limit") == null) ? 2048u : Convert.ToUInt32(Get(value2, "stream_limit"))),
					Workers = ((Get(value2, "worker_limit") == null) ? 40u : Convert.ToUInt32(Get(value2, "worker_limit"))),
					Name = Text(Get(item, "label")),
					Status = ((Text(Get(item, "role")) == "ru") ? "РФ" : "Выход") + " · " + ((num < 30.0) ? "В сети" : "Нет свежих сведений"),
					Address = Text(Get(item, "url"))
				});
			}
		}
		if (list.Items.Count > 0)
		{
			list.SelectedIndex = 0;
		}
		Say("Сведения получены через сеть. Прямая доступность может отличаться.");
	}

	private async Task SaveServer()
	{
		if (admin)
		{
			ServerRow row = C<ListBox>("Nodes").SelectedItem as ServerRow;
			if (row == null)
			{
				Say("Выберите сервер");
				return;
			}
			uint streams;
			uint workers;
			if (!uint.TryParse(C<TextBox>("StreamLimit").Text, out streams) || !uint.TryParse(C<TextBox>("WorkerLimit").Text, out workers) || streams < 1 || streams > 4096 || workers < 1 || workers > 4096)
			{
				Say("Введите значения от 1 до 4096");
				return;
			}
			await Cli(new string[2] { "admin", "network-set" }, json.Serialize(new
			{
				node_id = row.Id,
				stream_limit = streams,
				worker_limit = workers
			}));
			Say("Лимиты сохранены. Применение в течение нескольких секунд");
		}
	}

	private async Task Manage(string action)
	{
		if (!admin)
		{
			return;
		}
		string name = C<TextBox>("Username").Text.Trim();
		if (name.Length == 0)
		{
			Say("Укажите пользователя");
		}
		else
		{
			if (action == "remove" && MessageBox.Show(this.window, "Удалить выбранного пользователя?", "Garlic", MessageBoxButton.YesNo) != MessageBoxResult.Yes)
			{
				return;
			}
			Dictionary<string, object> body = new Dictionary<string, object>
			{
				{ "action", action },
				{ "username", name }
			};
			if (action == "create" || action == "subscription")
			{
				body["subscription"] = ((C<CheckBox>("Unlimited").IsChecked == true) ? "unlimited" : ((long)(DateTime.SpecifyKind(expiryDate.Date.AddDays(1.0).AddSeconds(-1.0), DateTimeKind.Local).ToUniversalTime() - new DateTime(1970, 1, 1)).TotalSeconds).ToString());
			}
			if (action == "create")
			{
				body["role"] = "client";
			}
			UserRow selected = C<ListBox>("Users").SelectedItem as UserRow;
			if (action == "subscription" && selected != null)
			{
				string origName = Text(Get(selected.Data, "username"));
				if (origName.Length > 0 && origName != name)
				{
					await Cli(new string[2] { "admin", "manage" }, json.Serialize(new Dictionary<string, object>
					{
						{ "action", "rename" },
						{ "username", origName },
						{ "new_username", name }
					}));
				}
			}
			if (action == "toggle")
			{
				body["action"] = ((selected != null && object.Equals(Get(selected.Data, "enabled"), true)) ? "disable" : "enable");
			}
			if (action == "role")
			{
				body["role"] = ((selected != null && Text(Get(selected.Data, "role")) == "admin") ? "client" : "admin");
			}
			Dictionary<string, object> result = await Cli(new string[2] { "admin", "manage" }, json.Serialize(body));
			string token = Text(Get(result, "token"));
			if (token.Length > 0)
			{
				Window window = new Window();
				window.Title = "Ключ пользователя";
				window.Width = 540.0;
				window.Height = 240.0;
				window.Owner = this.window;
				window.WindowStartupLocation = WindowStartupLocation.CenterOwner;
				Window window2 = window;
				StackPanel stackPanel = new StackPanel();
				stackPanel.Margin = new Thickness(24.0);
				StackPanel stackPanel2 = stackPanel;
				TextBox textBox = new TextBox();
				textBox.Text = token;
				textBox.IsReadOnly = true;
				textBox.TextWrapping = TextWrapping.Wrap;
				TextBox element = textBox;
				Button button = new Button();
				button.Content = "Скопировать";
				button.Margin = new Thickness(0.0, 20.0, 0.0, 0.0);
				button.Padding = new Thickness(12.0);
				Button button2 = button;
				button2.Click += (object sender, RoutedEventArgs e) =>
				{
					Clipboard.SetText(token);
				};
				stackPanel2.Children.Add(element);
				stackPanel2.Children.Add(button2);
				window2.Content = stackPanel2;
				window2.ShowDialog();
			}
			await Users();
		}
	}

	private GarlicApp(string baseVersion)
	{
		GarlicApp garlicApp = this;
		string baseDirectory = AppDomain.CurrentDomain.BaseDirectory;
		root = (File.Exists(Path.Combine(baseDirectory, "windows", "Interface.xaml")) ? baseDirectory : Path.GetFullPath(Path.Combine(baseDirectory, "..", "..")));
		string text = Path.Combine(root, "target", "windows-final", "release", "garlic-internal-network.exe");
		bundled = (File.Exists(Path.Combine(baseDirectory, "windows", "Interface.xaml")) ? Path.Combine(baseDirectory, "garlic-internal-network.exe") : (File.Exists(text) ? text : Path.Combine(root, "target", "release", "garlic-internal-network.exe")));
		binary = bundled;
		using (Stream stream = typeof(GarlicApp).Assembly.GetManifestResourceStream("Garlic.Interface.xaml"))
		{
			window = (Window)XamlReader.Load(stream);
		}
		string[] array = new string[]
		{
			"PluginsButton", "ChannelButton", "UpdateButton", "LogsButton", "HomeButton", "ServersButton", "NodeTitle", "UsersButton", "Logo", "Auth", "Home", "Admin", "Server",
			"Key", "Login", "Connection", "Subscription", "Power", "TunnelMode", "TunnelHint", "Route", "Takeover", "Logout",
			"Username", "Days", "Unlimited", "Create", "Issue", "ChangeSub", "Toggle", "RoleButton", "Remove", "Users",
			"Message", "RouteInfo", "Statistics", "ShowStatistics", "Servers", "Nodes", "StreamLimit", "WorkerLimit", "SaveServer", "RefreshServers",
			"LegacyUsersButton"
		};
		foreach (string text2 in array)
		{
			controls[text2] = window.FindName(text2);
		}
		string path = Path.Combine(root, "assets", "branding");
		C<Image>("Logo").Source = new BitmapImage(new Uri(Path.Combine(path, "garlic-logo-white.png")));
		window.Icon = BitmapFrame.Create(new Uri(Path.Combine(path, "garlic.ico")));
		ContextMenu tunnelMenu = new ContextMenu();
		tunnelMenu.Background = new SolidColorBrush(Color.FromRgb(41, 41, 41));
		tunnelMenu.Foreground = Brushes.White;
		tunnelMenu.BorderThickness = new Thickness(0.0);
		for (int j = 0; j < tunnelNames.Length; j++)
		{
			int selected = j;
			MenuItem menuItem = new MenuItem
			{
				Header = tunnelNames[j],
				Padding = new Thickness(16.0, 12.0, 16.0, 12.0),
				FontSize = 15.0
			};
			menuItem.Click += (object sender, RoutedEventArgs e) =>
			{
				garlicApp.ApplyTunnelMode(selected);
				if (garlicApp.connected)
				{
					garlicApp.Say("Новый режим подключения применится после переподключения");
				}
			};
			tunnelMenu.Items.Add(menuItem);
		}
#if GARLIC_ALPHA
		string alphaConfig = Environment.GetEnvironmentVariable("GARLIC_CLIENT_CONFIG");
		string slotFlag = Path.ChangeExtension(alphaConfig, "profile-disabled");
		MenuItem updates = new MenuItem {
			Header = "Обновлять профиль протокола", IsCheckable = true,
			IsChecked = !File.Exists(slotFlag)
		};
		updates.Click += delegate {
			try {
				Directory.CreateDirectory(Path.GetDirectoryName(slotFlag));
				if (updates.IsChecked) File.Delete(slotFlag);
				else File.WriteAllText(slotFlag, "");
			} catch (Exception ex) {
				updates.IsChecked = !File.Exists(slotFlag);
				garlicApp.Say(ex.Message);
			}
		};
		tunnelMenu.Items.Add(new Separator());
		tunnelMenu.Items.Add(updates);
		string startFlag = Path.ChangeExtension(alphaConfig, "auto-start");
		MenuItem startOnOpen = new MenuItem {
			Header = "Подключать при открытии", IsCheckable = true, IsChecked = File.Exists(startFlag)
		};
		startOnOpen.Click += delegate {
			try {
				Directory.CreateDirectory(Path.GetDirectoryName(startFlag));
				if (startOnOpen.IsChecked) File.WriteAllText(startFlag, "");
				else File.Delete(startFlag);
			} catch (Exception ex) {
				startOnOpen.IsChecked = File.Exists(startFlag);
				garlicApp.Say(ex.Message);
			}
		};
		tunnelMenu.Items.Add(startOnOpen);
#endif
		C<Button>("TunnelMode").ContextMenu = tunnelMenu;
		C<Button>("TunnelMode").Click += (object sender, RoutedEventArgs e) =>
		{
			tunnelMenu.PlacementTarget = garlicApp.C<Button>("TunnelMode");
			tunnelMenu.Placement = PlacementMode.Bottom;
			tunnelMenu.StaysOpen = false;
			tunnelMenu.IsOpen = true;
		};
		try
		{
			if (File.Exists(UiModePath()) && File.ReadAllText(UiModePath()).Trim() == "proxy")
			{
				ApplyTunnelMode(1, false);
			}
			else
			{
				ApplyTunnelMode(0, false);
			}
		}
		catch
		{
		}
		ContextMenu routeMenu = new ContextMenu();
		routeMenu.Background = new SolidColorBrush(Color.FromRgb(41, 41, 41));
		routeMenu.Foreground = Brushes.White;
		routeMenu.BorderThickness = new Thickness(0.0);
		string[] routeNames = new string[3] { "Авто: напрямую → РФ", "Только напрямую", "Только через Россию" };
		for (int j = 0; j < routeNames.Length; j++)
		{
			int selected2 = j;
			MenuItem menuItem = new MenuItem
			{
				Header = routeNames[j],
				Padding = new Thickness(16.0, 12.0, 16.0, 12.0),
				FontSize = 15.0
			};
			menuItem.Click += (object sender, RoutedEventArgs e) =>
			{
				garlicApp.routeMode = selected2;
				garlicApp.C<Button>("Route").Content = routeNames[selected2] + "  ⌄";
				if (garlicApp.connected)
				{
					garlicApp.Say("Новый маршрут применится после переподключения");
				}
			};
			routeMenu.Items.Add(menuItem);
		}
		C<Button>("Route").ContextMenu = routeMenu;
		C<Button>("Route").Click += (object sender, RoutedEventArgs e) =>
		{
			routeMenu.PlacementTarget = garlicApp.C<Button>("Route");
			routeMenu.Placement = PlacementMode.Bottom;
			routeMenu.StaysOpen = false;
			routeMenu.IsOpen = true;
		};
		DateLabel();
		C<Button>("Days").Click += (object sender, RoutedEventArgs e) =>
		{
			Calendar();
		};
		C<CheckBox>("Unlimited").Checked += (object sender, RoutedEventArgs e) =>
		{
			C<Button>("Days").IsEnabled = false;
		};
		C<CheckBox>("Unlimited").Unchecked += (object sender, RoutedEventArgs e) =>
		{
			C<Button>("Days").IsEnabled = true;
		};
		#if GARLIC_BUNDLE
        foreach (string nav in new[]{"HomeButton","UsersButton","ServersButton"}) {
            C<Button>(nav).Click += delegate {
                var host=(Grid)C<FrameworkElement>("Auth").Parent;
                var p=host.Children.OfType<ScrollViewer>().FirstOrDefault(x=>Equals(x.Tag,"plugins"));
                if(p!=null)host.Children.Remove(p);
            };
        }
        C<Button>("PluginsButton").Click += delegate {
            PluginFiles.Show(window, C<Button>("PluginsButton"), Environment.GetEnvironmentVariable("GARLIC_CLIENT_CONFIG"), Say, async (id,url,key) => { await Cli(id==null?new[]{"plugin-key",url,"--key-stdin"}:new[]{"plugin-slot-key",id,url,"--key-stdin"},key); });
        };
        var channelButton = C<Button>("ChannelButton");
        channelButton.Visibility = Visibility.Visible;
        channelButton.Click += delegate {
            var menu = new ContextMenu();
            foreach (bool previews in new[] { false, true }) {
                bool selected = previews;
                var item = new MenuItem { Header = selected ? "Релизы и пререлизы" : "Стабильные релизы", IsChecked = ReleaseClient.Previews == selected };
                item.Click += delegate {
                    try { ReleaseClient.SetPreviews(selected); }
                    catch (Exception error) { MessageBox.Show(window,error.Message,"Источник обновлений"); }
                };
                menu.Items.Add(item);
            }
            menu.PlacementTarget = channelButton;
            menu.IsOpen = true;
        };
        C<Button>("UpdateButton").Visibility = Visibility.Visible;
        C<Button>("UpdateButton").Click += delegate { ReleaseClient.Check(window, true); };
#endif
        Click("LogsButton", () =>
		{
			string text3 = Path.Combine(UiDataDirectory(), "logs");
			Directory.CreateDirectory(text3);
			Process.Start(new ProcessStartInfo("explorer.exe", "\"" + text3 + "\"")
			{
				UseShellExecute = true
			});
			return Task.FromResult(0);
		});
		Click("Login", Login);
		Click("Power", async () =>
		{
			if (changingConnection)
			{
				return;
			}
			changingConnection = true;
			C<Button>("Power").IsEnabled = false;
			try
			{
				if (connected || pending)
				{
					await StopVpn();
				}
				else
				{
					await StartVpn();
				}
			}
			finally
			{
				changingConnection = false;
				C<Button>("Power").IsEnabled = true;
			}
		});
		Click("Takeover", async () =>
		{
			if (changingConnection || signingOut)
			{
				return;
			}
			changingConnection = true;
			int request = accountGeneration;
			try
			{
				await StopVpn();
				if (request == accountGeneration)
				{
					await Cli(new string[1] { "takeover" });
					if (request == accountGeneration)
					{
						await StartVpn();
					}
				}
			}
			finally
			{
				changingConnection = false;
			}
		});
		Click("Logout", async () =>
		{
			if (signingOut)
			{
				return;
			}
			signingOut = true;
			try
			{
				accountGeneration++;
				await Cli(new string[1] { "signout" });
				savedAccount = null;
				admin = false;
				C<ListBox>("Users").Items.Clear();
				C<ListBox>("Nodes").Items.Clear();
				C<FrameworkElement>("Servers").Visibility = Visibility.Collapsed;
				C<FrameworkElement>("Home").Visibility = Visibility.Collapsed;
				C<FrameworkElement>("Admin").Visibility = Visibility.Collapsed;
				C<Button>("UsersButton").Visibility = Visibility.Collapsed;
				C<Button>("ServersButton").Visibility = Visibility.Collapsed;
				C<FrameworkElement>("Auth").Visibility = Visibility.Visible;
				Say("");
			}
			finally
			{
				signingOut = false;
			}
		});
		Click("LegacyUsersButton", LegacyUsers);
		Click("UsersButton", Users);
		Click("ServersButton", Servers);
		Click("HomeButton", () =>
		{
			if (savedAccount != null)
			{
				Home(savedAccount);
			}
			Say("");
			return Task.FromResult(0);
		});
		Click("RefreshServers", Servers);
		Click("SaveServer", SaveServer);
		C<ListBox>("Nodes").SelectionChanged += (object sender, SelectionChangedEventArgs e) =>
		{
			ServerRow serverRow = C<ListBox>("Nodes").SelectedItem as ServerRow;
			if (serverRow != null)
			{
				C<Button>("SaveServer").IsEnabled = true;
				C<TextBlock>("NodeTitle").Text = serverRow.Name;
				C<TextBox>("StreamLimit").Text = serverRow.Streams.ToString();
				C<TextBox>("WorkerLimit").Text = serverRow.Workers.ToString();
			}
		};
		C<ListBox>("Users").SelectionChanged += (object sender, SelectionChangedEventArgs e) =>
		{
			UserRow userRow = C<ListBox>("Users").SelectedItem as UserRow;
			if (userRow != null)
			{
				C<TextBox>("Username").Text = Text(Get(userRow.Data, "username"));
				object value = Get(userRow.Data, "subscription");
				C<CheckBox>("Unlimited").IsChecked = Text(Get(value, "kind")) == "unlimited";
				object obj2 = Get(value, "expires_at");
				expiryDate = ((obj2 == null) ? DateTime.Today.AddMonths(1) : new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc).AddSeconds(Convert.ToDouble(obj2)).ToLocalTime().Date);
				DateLabel();
			}
		};
		Click("Create", () => Manage("create"));
		Click("Issue", () => Manage("issue_key"));
		Click("ChangeSub", () => Manage("subscription"));
		Click("Toggle", () => Manage("toggle"));
		Click("RoleButton", () => Manage("role"));
		Click("Remove", () => Manage("remove"));
		string statsPath = Path.Combine(UiDataDirectory(), "client.show-statistics.txt");
		C<CheckBox>("ShowStatistics").IsChecked = File.Exists(statsPath) && File.ReadAllText(statsPath).Trim() == "1";
		Action updateStatistics = () =>
		{
			bool flag = garlicApp.C<CheckBox>("ShowStatistics").IsChecked == true;
			garlicApp.C<TextBlock>("Statistics").Visibility = ((!flag) ? Visibility.Collapsed : Visibility.Visible);
			try
			{
				Directory.CreateDirectory(Path.GetDirectoryName(statsPath));
				File.WriteAllText(statsPath, flag ? "1" : "0");
			}
			catch
			{
			}
		};
		C<CheckBox>("ShowStatistics").Checked += (object s, RoutedEventArgs e) =>
		{
			updateStatistics();
		};
		C<CheckBox>("ShowStatistics").Unchecked += (object s, RoutedEventArgs e) =>
		{
			updateStatistics();
		};
		updateStatistics();
		timer.Interval = TimeSpan.FromSeconds(2.0);
		timer.Tick += async (object sender, EventArgs e) =>
		{
			await Poll();
		};
		bool initialized = false;
		window.Loaded += async (object sender, RoutedEventArgs e) =>
		{
			if (initialized) return;
			initialized = true;
			try
			{
				garlicApp.Say("Запускаем сетевой модуль…");
				var coreInfo = await garlicApp.Cli(new string[] { "version" }, null, 60000);
#if GARLIC_ALPHA
                if (Text(Get(coreInfo, "core_abi")) != "1" || !object.Equals(Get(coreInfo, "alpha_profiles"), true))
#else
                if (Text(Get(coreInfo, "version")) != baseVersion)
#endif
				{
					throw new Exception("Версии приложения и сетевого модуля различаются");
				}
#if GARLIC_ALPHA
				garlicApp.window.Title = "Garlic Alpha";
#else
				try
				{
					await garlicApp.SelectCore();
				}
				catch (Exception)
				{
				}
				garlicApp.window.Title = "Garlic · ядро " + Text(Get(await garlicApp.Cli(new string[1] { "version" }), "version"));
				garlicApp.StageCore();
				garlicApp.StageInterface();
#endif
				garlicApp.Say("");
			}
			catch (Exception ex2)
			{
				string[] array2 = new string[5] { "Login", "Power", "UsersButton", "ServersButton", "Takeover" };
				foreach (string name in array2)
				{
					garlicApp.C<Button>(name).IsEnabled = false;
				}
				if (true)
				{
				}
				garlicApp.Say("Не удалось запустить сетевой модуль. " + ex2.Message);
				return;
			}
			string cache = Path.Combine(UiDataDirectory(), "client.account-cache.json");
			if (File.Exists(cache))
			{
				try
				{
					garlicApp.Home(garlicApp.json.Deserialize<Dictionary<string, object>>(File.ReadAllText(cache)));
				}
				catch
				{
				}
			}
			try
			{
				Dictionary<string, object> result = await garlicApp.Cli(new string[1] { "account" });
				garlicApp.Home(Get(result, "account"));
			}
			catch
			{
				if (File.Exists(cache))
				{
					garlicApp.Say("Аккаунт сохранён. Сервер временно недоступен или ключ требует проверки");
				}
			}
			garlicApp.timer.Start();
#if GARLIC_ALPHA
			if (File.Exists(startFlag) && garlicApp.savedAccount != null && !garlicApp.changingConnection) {
				garlicApp.changingConnection = true;
				try {
					await garlicApp.Poll();
					if (!garlicApp.connected && !garlicApp.pending) await garlicApp.StartVpn();
				} catch (Exception ex) {
					garlicApp.Say(ex.Message);
				} finally {
					garlicApp.changingConnection = false;
				}
			}
#endif
		};
		window.Closed += (object sender, EventArgs e) =>
		{
			timer.Stop();
			if (coreLock != null)
			{
				coreLock.Dispose();
			}
		};
	}

	public static Window CreateWindow(string baseVersion)
	{
		return new GarlicApp(baseVersion).window;
	}

	private static IEnumerable<object> LegacyArray(object value)
	{
		return ((value as IEnumerable) ?? new object[0]).Cast<object>();
	}

	private async Task LegacyUsers()
	{
		int session = accountGeneration;
		if (!admin)
		{
			return;
		}
		Window dialog = new Window
		{
			Owner = window,
			Title = "Пользователи Legacy",
			Width = 620.0,
			Height = 630.0,
			WindowStartupLocation = WindowStartupLocation.CenterOwner
		};
		StackPanel box = new StackPanel
		{
			Margin = new Thickness(14.0)
		};
		dialog.Content = new ScrollViewer
		{
			Content = box,
			VerticalScrollBarVisibility = ScrollBarVisibility.Auto
		};
		ListBox users = new ListBox
		{
			Height = 205.0
		};
		box.Children.Add(users);
		TextBox days = new TextBox
		{
			Text = "30"
		};
		TextBox name = new TextBox();
		DatePicker expiry = new DatePicker
		{
			SelectedDate = DateTime.Today.AddMonths(1)
		};
		CheckBox unlimited = new CheckBox
		{
			Content = "Без срока"
		};
		WrapPanel routes = new WrapPanel();
		TextBlock status = new TextBlock
		{
			TextWrapping = TextWrapping.Wrap
		};
		Dictionary<int, CheckBox> choices = new Dictionary<int, CheckBox>();
		string origin = null;
		bool busy = false;
		Func<bool> active = () => admin && session == accountGeneration && dialog.IsVisible;
		Func<object, Task<Dictionary<string, object>>> invoke = async (object op) =>
		{
			Dictionary<string, object> request = new Dictionary<string, object>();
			if (op != null)
			{
				request["origin"] = origin;
				request["operation"] = op;
			}
			Dictionary<string, object> result = await Cli(new string[2] { "admin", "network-set" }, json.Serialize(new
			{
				legacy = request
			}));
			if (!object.Equals(Get(result, "ok"), true))
			{
				throw new Exception("T1 не подтвердил запрос. Обновите список перед повтором.");
			}
			return result;
		};
		Func<Task> refresh = async () =>
		{
			Dictionary<string, object> response = await invoke(null);
			if (!active())
			{
				return;
			}
			origin = Text(Get(response, "origin"));
			object directory = Get(response, "legacy");
			Dictionary<int, string> labels = new Dictionary<int, string>();
			choices.Clear();
			routes.Children.Clear();
			users.Items.Clear();
			foreach (object item in LegacyArray(Get(directory, "routes")))
			{
				int key = Convert.ToInt32(Get(item, "id"));
				string text = Text(Get(item, "protocol"));
				string text2 = text + " / " + Text(Get(item, "port"));
				labels[key] = text2;
				if (!(Text(Get(item, "nodeId")) != Text(Get(directory, "worker"))) && object.Equals(Get(item, "enable"), true) && new string[3] { "vless", "vmess", "trojan" }.Contains(text))
				{
					CheckBox checkBox = new CheckBox
					{
						Content = text2,
						Margin = new Thickness(4.0)
					};
					choices[key] = checkBox;
					routes.Children.Add(checkBox);
				}
			}
			DateTime epoch = new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc);
			foreach (object item2 in LegacyArray(Get(directory, "users")))
			{
				long num = Convert.ToInt64(Get(item2, "expiryTime"));
				string text3 = Text(Get(item2, "email"));
				bool flag = object.Equals(Get(item2, "enable"), true);
				object obj = Get(item2, "traffic");
				string text4 = ((num == 0) ? "Без срока" : ((num < 0) ? "Срок с первого подключения" : ("До " + epoch.AddMilliseconds(num).ToLocalTime().ToShortDateString())));
				IEnumerable<string> values = from i in LegacyArray(Get(item2, "inboundIds"))
					select labels.ContainsKey(Convert.ToInt32(i)) ? labels[Convert.ToInt32(i)] : ("#" + Text(i));
				string text5 = ((obj == null) ? "" : string.Format(" · {0:N1} МиБ", (Convert.ToDouble(Get(obj, "up")) + Convert.ToDouble(Get(obj, "down"))) / 1048576.0));
				users.Items.Add(new LegacyEntry
				{
					Email = text3,
					Label = text3 + (flag ? "" : " · отключён") + " · " + text4 + text5 + "\n" + string.Join(", ", values)
				});
			}
		};
		Action<Panel, string, Func<Task>> button = (Panel parent, string title, Func<Task> action) =>
		{
			Button button2 = new Button
			{
				Content = title,
				Margin = new Thickness(3.0),
				Padding = new Thickness(8.0, 4.0, 8.0, 4.0)
			};
			parent.Children.Add(button2);
			button2.Click += async (object s, RoutedEventArgs e) =>
			{
				if (busy || !active())
				{
					return;
				}
				busy = true;
				box.IsEnabled = false;
				try
				{
					status.Text = "";
					await action();
				}
				catch (Exception ex)
				{
					status.Text = ex.Message;
				}
				finally
				{
					box.IsEnabled = true;
					busy = false;
				}
			};
		};
		Func<LegacyEntry> selected = () =>
		{
			LegacyEntry legacyEntry = users.SelectedItem as LegacyEntry;
			if (legacyEntry == null)
			{
				throw new Exception("Выберите пользователя");
			}
			return legacyEntry;
		};
		WrapPanel commands = new WrapPanel();
		box.Children.Add(commands);
		button(commands, "Обновить", refresh);
		button(commands, "Ссылка подписки", async () =>
		{
			LegacyEntry u = selected();
			Dictionary<string, object> r = await invoke(new
			{
				action = "subscription",
				email = u.Email
			});
			if (active() && MessageBox.Show(dialog, "Скопировать ссылку подписки для " + u.Email + "?", "Подписка T1", MessageBoxButton.YesNo) == MessageBoxResult.Yes)
			{
				Clipboard.SetText(Text(Get(r, "url")));
			}
		});
		box.Children.Add(new TextBlock
		{
			Text = "Продлить выбранную подписку на количество дней"
		});
		box.Children.Add(days);
		button(box, "Продлить", async () =>
		{
			LegacyEntry u = selected();
			int n;
			if (!int.TryParse(days.Text, out n) || n < 1 || n > 3650)
			{
				throw new Exception("Введите от 1 до 3650 дней");
			}
			await invoke(new
			{
				action = "extend",
				email = u.Email,
				days = n
			});
			await refresh();
		});
		box.Children.Add(new TextBlock
		{
			Text = "Имя нового пользователя"
		});
		box.Children.Add(name);
		box.Children.Add(new TextBlock
		{
			Text = "Срок включительно"
		});
		box.Children.Add(expiry);
		box.Children.Add(unlimited);
		box.Children.Add(new TextBlock
		{
			Text = "Входы W"
		});
		box.Children.Add(routes);
		button(box, "Создать пользователя", async () =>
		{
			int[] ids = (from k in choices
				where k.Value.IsChecked == true
				select k.Key).ToArray();
			if (ids.Length == 0 || name.Text.Trim().Length == 0 || (unlimited.IsChecked != true && !expiry.SelectedDate.HasValue))
			{
				throw new Exception("Введите имя, срок и выберите вход W");
			}
			long ms = ((unlimited.IsChecked == true) ? 0 : ((long)(expiry.SelectedDate.Value.Date.AddDays(1.0).ToUniversalTime() - new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc)).TotalMilliseconds - 1));
			Dictionary<string, object> r = await invoke(new
			{
				action = "create",
				email = name.Text.Trim(),
				expiry_ms = ms,
				inbound_ids = ids
			});
			await refresh();
			status.Text = (object.Equals(Get(r, "nodePending"), true) ? "Создан на T1. W ожидает синхронизации" : "Пользователь создан");
		});
		box.Children.Add(status);
		dialog.Loaded += async (object s, RoutedEventArgs e) =>
		{
			busy = true;
			box.IsEnabled = false;
			try
			{
				await refresh();
			}
			catch (Exception ex)
			{
				status.Text = ex.Message;
			}
			finally
			{
				box.IsEnabled = true;
				busy = false;
			}
		};
		dialog.ShowDialog();
		await Task.FromResult(0);
	}
}
internal static class GarlicBuild
{
	#if GARLIC_ALPHA
    internal const string Version = "0.9.13";
#else
    internal const string Version = "0.9.2";
#endif
}
internal static class InterfaceBootstrap
{
	private static Dictionary<string, object> Update(params string[] args)
	{
		string fileName = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "garlic-ui-update.exe");
		ProcessStartInfo processStartInfo = new ProcessStartInfo(fileName, string.Join(" ", args));
		processStartInfo.UseShellExecute = false;
		processStartInfo.CreateNoWindow = true;
		processStartInfo.RedirectStandardOutput = true;
		ProcessStartInfo startInfo = processStartInfo;
		using (Process process = Process.Start(startInfo))
		{
			Task<string> task = process.StandardOutput.ReadToEndAsync();
			if (!process.WaitForExit(15000))
			{
				process.Kill();
				throw new IOException("UI update timed out");
			}
			if (process.ExitCode != 0)
			{
				throw new IOException("UI update selection failed");
			}
			return new JavaScriptSerializer().Deserialize<Dictionary<string, object>>(task.GetAwaiter().GetResult());
		}
	}

	private static Window SelectWindow()
	{
#if !GARLIC_ALPHA
		for (int i = 0; i < 2; i++)
		{
			bool flag = false;
			try
			{
				Dictionary<string, object> dictionary = Update("select");
				if (!dictionary.ContainsKey("path"))
				{
					break;
				}
				flag = object.Equals(dictionary["trial"], true);
				byte[] array;
				using (FileStream fileStream = File.OpenRead((string)dictionary["path"]))
				{
					if (fileStream.Length <= 0 || fileStream.Length > 8388608)
					{
						throw new IOException("UI size");
					}
					using (BinaryReader binaryReader = new BinaryReader(fileStream))
					{
						array = binaryReader.ReadBytes((int)fileStream.Length);
					}
				}
				using (SHA256 sHA = SHA256.Create())
				{
					string text = BitConverter.ToString(sHA.ComputeHash(array)).Replace("-", "").ToLowerInvariant();
					if (text != (string)dictionary["sha256"])
					{
						throw new IOException("UI digest");
					}
				}
				Type type = Assembly.Load(array).GetType("GarlicApp", true);
				MethodInfo method = type.GetMethod("CreateWindow", new Type[1] { typeof(string) });
				Window window = (Window)method.Invoke(null, new object[1] { GarlicBuild.Version });
				if (window == null)
				{
					throw new InvalidOperationException("UI returned no window");
				}
				if (flag)
				{
					string sequence = Convert.ToUInt64(dictionary["sequence"]).ToString(CultureInfo.InvariantCulture);
					EventHandler ready = null;
					ready = (object sender, EventArgs e) =>
					{
						window.ContentRendered -= ready;
						try
						{
							Update("confirm", sequence);
						}
						catch (Exception)
						{
						}
					};
					window.ContentRendered += ready;
				}
				return window;
			}
			catch (Exception)
			{
				if (!flag)
				{
					break;
				}
				continue;
			}
		}
#endif
		return GarlicApp.CreateWindow(GarlicBuild.Version);
	}

	[STAThread]
	public static void Main()
	{
		try
		{
			Environment.SetEnvironmentVariable("GARLIC_APP_VERSION", GarlicBuild.Version);
#if GARLIC_ALPHA
            if (string.IsNullOrEmpty(Environment.GetEnvironmentVariable("GARLIC_CLIENT_CONFIG")))
                Environment.SetEnvironmentVariable("GARLIC_CLIENT_CONFIG", Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Garlic Alpha", "client.json"));
#endif
#if GARLIC_BUNDLE
            if (!ReleaseClient.Prepare()) return;
#endif
            GarlicApp.LogUi("app_start", "windows");
            Application application = new Application();
			string path = Environment.GetEnvironmentVariable("GARLIC_CLIENT_CONFIG") ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Garlic", "client.json");
			string text;
			using (SHA256 sHA = SHA256.Create())
			{
				text = BitConverter.ToString(sHA.ComputeHash(Encoding.UTF8.GetBytes(Path.GetFullPath(path).ToUpperInvariant())));
			}
			bool createdNew;
			#if GARLIC_BUNDLE
            using (var signal = new EventWaitHandle(false, EventResetMode.AutoReset, "Local\\GarlicShow-" + text))
            using (Mutex mutex = new Mutex(true, "Local\\GarlicTrayInterface-" + text, out createdNew))
#else
            using (Mutex mutex = new Mutex(true, "Local\\GarlicInterface-" + text, out createdNew))
#endif
			{
				try
				{
#if GARLIC_BUNDLE
                    if (!createdNew) {
                        signal.Set();
                        return;
                    }
                    Window main = SelectWindow();
                    main.ContentRendered += delegate { ReleaseClient.Check(main); };
                    using (var tray = new TrayWindow(main, path, signal)) application.Run(main);
#else
                    Window main = createdNew ? SelectWindow() : GarlicApp.CreateWindow(GarlicBuild.Version);
                    application.Run(main);
#endif
				}
				finally
				{
					if (createdNew)
					{
						mutex.ReleaseMutex();
					}
				}
			}
		}
		catch (Exception ex)
		{
			MessageBox.Show(ex.Message, "Garlic");
		}
	}
}
