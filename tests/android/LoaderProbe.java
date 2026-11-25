package garlic.lab;

import android.content.Context;
import android.content.ContextWrapper;
import android.content.SharedPreferences;
import android.content.pm.ApplicationInfo;

import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import org.json.JSONObject;

/** app_process smoke test. Use the release APK on CLASSPATH and its extracted ABI libraries. */
public final class LoaderProbe {
    public static void main(String[] args) throws Exception {
        ApplicationInfo info = new ApplicationInfo();
        info.nativeLibraryDir = args[0];
        Context context = new ContextWrapper(null) {
            @Override public ApplicationInfo getApplicationInfo() { return info; }
            @Override public SharedPreferences getSharedPreferences(String name, int mode) {
                return (SharedPreferences) Proxy.newProxyInstance(LoaderProbe.class.getClassLoader(),
                    new Class<?>[]{SharedPreferences.class}, (proxy, method, values) -> {
                        if (method.getName().equals("getString") || method.getName().equals("getBoolean"))
                            return values[1];
                        throw new UnsupportedOperationException(method.getName());
                    });
            }
        };
        Class<?> loader = Class.forName("org.garlic.client.CoreLoader");
        Method initialize = loader.getDeclaredMethod("initialize", Context.class);
        initialize.setAccessible(true);
        initialize.invoke(null, context);
        Class<?> bridge = Class.forName("org.garlic.client.GarlicBridge");
        Object error = bridge.getField("loadError").get(null);
        if (error != null) {
            System.out.println("LOAD ERROR " + error);
            if (args.length == 2 && error.toString().contains(args[1])) return;
            throw new AssertionError(error);
        }
        if (args.length != 1) throw new AssertionError("Expected the previous APK to fail");
        new JSONObject((String) bridge.getMethod("status").invoke(null));
        System.out.println("PASS " + android.os.Build.SUPPORTED_ABIS[0] + " CoreLoader.initialize and GarlicBridge.status");
    }
}
