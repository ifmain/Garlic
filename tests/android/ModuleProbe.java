package garlic.lab;

import android.app.Instrumentation;
import android.os.Bundle;
import android.system.Os;
import java.io.File;
import java.nio.file.Files;
import java.util.Arrays;
import org.json.JSONObject;

public final class ModuleProbe extends Instrumentation {
    private final java.util.concurrent.CountDownLatch initialized = new java.util.concurrent.CountDownLatch(1);
    private Bundle args;
    @Override public void onCreate(Bundle extras) { args = extras; start(); }
    @Override public void callApplicationOnCreate(android.app.Application app) {
        super.callApplicationOnCreate(app);
        initialized.countDown();
    }
    private void call(String method, String... values) throws Exception {
        Class<?> bridge = getTargetContext().getClassLoader().loadClass("org.garlic.client.GarlicBridge");
        Class<?>[] types = new Class<?>[values.length];
        Arrays.fill(types, String.class);
        JSONObject reply = new JSONObject((String)bridge.getMethod(method,types).invoke(null,(Object[])values));
        if (!reply.optBoolean("ok")) throw new Exception(method + ": " + reply.optString("error"));
    }
    @Override public void onStart() {
        Bundle result = new Bundle();
        try {
            if (!initialized.await(10, java.util.concurrent.TimeUnit.SECONDS)) throw new Exception("Application startup timed out");
            File dir = Files.createTempDirectory(getTargetContext().getCacheDir().toPath(),"module-test").toFile();
            for (String name : getContext().getAssets().list("fixture")) {
                try (java.io.InputStream source = getContext().getAssets().open("fixture/"+name)) {
                    Files.copy(source,new File(dir,name).toPath());
                }
            }
            Os.setenv("GARLIC_CA_CERT",new File(dir,"ca.pem").toString(),true);
            String origin = args.getString("origin");
            call("loginMulti",dir.toString(),origin,"fixture",args.getString("secret"));
            String direct = args.getString("direct");
            File sessions = new File(dir, "client.sessions.json");
            JSONObject book = new JSONObject(new String(Files.readAllBytes(sessions.toPath()), "UTF-8"));
            JSONObject entries = book.getJSONObject("entries");
            String ingress = args.getString("ingress");
            entries.put(ingress, entries.getJSONObject(origin));
            entries.put(direct, entries.getJSONObject(origin));
            Files.write(sessions.toPath(), book.toString().getBytes("UTF-8"));
            call("startProxy",dir.toString(),"auto",direct,ingress,args.getString("bind"));
            Bundle ready = new Bundle(); ready.putString("state","proxy ready"); sendStatus(1,ready);
            Thread.sleep(120000);
            call("stopProxy");
            finish(-1,result);
        } catch (Throwable e) {
            result.putString("error",android.util.Log.getStackTraceString(e)); finish(0,result);
        }
    }
}
