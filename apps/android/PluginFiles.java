package org.garlic.client;

import android.app.*;
import android.content.*;
import android.widget.Toast;
import android.util.AtomicFile;
import java.io.*;
import java.nio.charset.StandardCharsets;
import org.json.*;

public final class PluginFiles {
    static final int INSTALL = 4851, IMPORT = 4852, EXPORT = 4853;
    static File folder(Activity a) {
        String id=a instanceof PluginPage?((PluginPage)a).selected:null;
        return folder(a,id);
    }
    static File folder(Activity a,String id){
        File base=new File(a.getFilesDir(),"garlic");
        if(id==null)return base;
        if(!id.matches("[a-z0-9-]{1,48}"))throw new IllegalArgumentException();
        return new File(base,"plugins-v3/s-"+id);
    }
    static File state(Activity a){return new File(folder(a),"client.installed-gpf");}
    static File module(Activity a) { return new File(folder(a),"client.gpf-module"); }
    static JSONObject manifest(Activity a) throws Exception {
        return new JSONObject(read(new AtomicFile(module(a)).openRead()));
    }
    static String title(Activity a) {
        try { return manifest(a).getString("name"); } catch(Exception e) { return "Плагин"; }
    }
    static synchronized void install(Activity a,String text) throws Exception {
        JSONObject envelope=new JSONObject(text);
        byte[] raw=android.util.Base64.decode(envelope.getString("payload"),0);
        byte[] sig=android.util.Base64.decode(envelope.getString("signature"),0);
        java.math.BigInteger modulus=new java.math.BigInteger(1,android.util.Base64.decode("yKZsHlxyYYIQpLC7Q8ULIVenam4A5M67WawO1iNoxS0LzJInrJgpd68lyNpze2chK4TgY+gVhSB1eCm5PrvaD1sJj4O4xkD4u3vrOnoeVzxJNx1Yl4pq9tE1RzaEOxO84WIdoZt+gl2IALI23aXYBKGqkVTsKIlihFEmpR9f473eomBiRh8KR1qVK0/J1K2QChzjqxGg3f0NmDE+yQHXLH2JUq7gnwvydedB0ZK+xpnoHpIkjt8jwQNa6Trj3JK1rQfoVEzkw/m2VYTcGnQ8taN3amdhyGFNLei4onQ7nwVO1HyAB7DTZ9ZOW8hF0B97ucH4062W9pHnDv51DHai7wmr8By5agBSgnryqK4sDPzYAN30ryF5mef0A/gRlITbzMU+W0cjPxzMwgCk/Q5nct40h5G5d06Yg/W8ZIsTrliD7bVco1EeVLoVrdab55+hbRwiF+AMa78TilyZ+hfXXfyz+Nu9P2hRN+eBzNIAlXZt6loL8EJY/G/ijfs01tKF",0));
        java.security.Signature verifier=java.security.Signature.getInstance("SHA256withRSA");
        verifier.initVerify(java.security.KeyFactory.getInstance("RSA").generatePublic(new java.security.spec.RSAPublicKeySpec(modulus,java.math.BigInteger.valueOf(65537))));
        verifier.update(raw);
        if(!verifier.verify(sig))throw new IOException("signature");
        JSONObject m=new JSONObject(new String(raw,StandardCharsets.UTF_8));
        if(!"gpf/2".equals(m.getString("format")) || m.getString("wasm").length()>8192 || m.getString("name").length()>80)throw new IOException("module");
        JSONObject opts=new JSONObject().put("format","garlic-plugin-data/1").put("id",m.getString("id")).put("enabled",true)
            .put("interval_seconds",m.getInt("interval_seconds")).put("ingress_penalty_ms",m.getInt("ingress_penalty_ms"));
        opts=parse(opts.toString());
        if(state(a).isFile()){JSONObject prior=load(a);if(!prior.getString("id").equals(opts.getString("id")))throw new IOException("slot occupied");opts=prior;}
        save(a,opts);
        AtomicFile dest=new AtomicFile(module(a));FileOutputStream stream=dest.startWrite();
        try{stream.write(raw);dest.finishWrite(stream);}catch(Exception e){dest.failWrite(stream);throw e;}
    }
    static void importPackage(PluginPage a,String text) throws Exception {
        JSONObject env=new JSONObject(text);
        if(!env.has("author")){a.selected=null;install(a,text);a.render();return;}
        byte[] modulus=android.util.Base64.decode(env.getString("author"),0), raw=android.util.Base64.decode(env.getString("payload"),0);
        if(modulus.length!=384 || (modulus[0]&128)==0)throw new IOException();
        java.security.Signature verifier=java.security.Signature.getInstance("SHA256withRSA");
        verifier.initVerify(java.security.KeyFactory.getInstance("RSA").generatePublic(new java.security.spec.RSAPublicKeySpec(new java.math.BigInteger(1,modulus),java.math.BigInteger.valueOf(65537))));
        verifier.update(raw);
        if(!verifier.verify(android.util.Base64.decode(env.getString("signature"),0)))throw new IOException();
        JSONObject m=new JSONObject(new String(raw,StandardCharsets.UTF_8));
        String id=m.getString("id"),cap=m.getString("capability"),name=m.getString("name");
        if(!("gpf/3".equals(m.getString("format")) || ("gpf/4".equals(m.getString("format")) && m.optInt("wire_abi")==1 && "network".equals(cap))) || !("canvas".equals(cap)||"network".equals(cap)) ||
            !id.matches("[a-z0-9-]{1,48}") || name.isEmpty() || name.length()>80 || m.getString("wasm").length()>8192)throw new IOException();
        StringBuilder fp=new StringBuilder();for(byte b:java.security.MessageDigest.getInstance("SHA-256").digest(modulus))fp.append(String.format(java.util.Locale.ROOT,"%02X",b));
        File file=new File(folder(a,id),"client.gpf-module");
        if(file.exists() && !new JSONObject(read(new AtomicFile(file).openRead())).getString("publisher").equals(fp.toString()))throw new IOException("publisher mismatch");
        new AlertDialog.Builder(a).setTitle("Установить плагин?").setMessage(name+" ("+id+")\n"+("canvas".equals(cap)?"Экран и ввод на нём":"gpf/4".equals(m.optString("format"))?"Обработка трафика слота и данных входа выбранного подключения":"Выбор разрешённых подключений")+"\nSHA-256: "+fp)
            .setNegativeButton("Отмена",null).setPositiveButton("Установить",(dialog,which)->{
                try{
                    a.selected=id;
                    if(!state(a).isFile())save(a,new JSONObject().put("format","garlic-plugin-data/1").put("id",id).put("enabled",true)
                        .put("connections",new JSONArray()).put("interval_seconds",5).put("ingress_penalty_ms",0));
                    m.put("publisher",fp.toString());AtomicFile dest=new AtomicFile(file);FileOutputStream out=dest.startWrite();
                    try{out.write(m.toString().getBytes(StandardCharsets.UTF_8));dest.finishWrite(out);}catch(Exception e){dest.failWrite(out);throw e;}
                }catch(Exception e){toast(a,"Установка не удалась");}finally{a.selected=null;a.render();}
            }).show();
    }
    static String address(String value) throws JSONException { return address(value,"direct"); }
    static String address(String value,String kind) throws JSONException {
        try {
            java.net.URI u=new java.net.URI(value);
            if (!(kind.equals("direct") || kind.equals("ingress") ? "https" : kind).equals(u.getScheme()) || u.getHost()==null || u.getUserInfo()!=null || u.getQuery()!=null ||
                u.getFragment()!=null || !(u.getPath().isEmpty()||u.getPath().equals("/")) || u.getPort()>65535 || u.getPort()==0 || value.length()>240)
                throw new Exception();
            return value.endsWith("/")?value.substring(0,value.length()-1):value;
        } catch(Exception e){throw new JSONException("address");}
    }
    static JSONObject parse(String text) throws JSONException {
        JSONTokener input = new JSONTokener(text);
        Object item = input.nextValue();
        if (!(item instanceof JSONObject) || input.nextClean() != 0) throw new JSONException("JSON");
        JSONObject v = (JSONObject)item;
        if (v.length() != (v.has("connections") ? 6 : 5) || !v.getString("id").matches("[a-z0-9-]{1,48}") ||
            !"garlic-plugin-data/1".equals(v.get("format")) ||
            !(v.get("enabled") instanceof Boolean)) throw new JSONException("Format");
        for (String key : new String[]{"interval_seconds","ingress_penalty_ms"}) {
            Object n = v.get(key);
            boolean interval = key.equals("interval_seconds");
            if (!(n instanceof Integer) || (Integer)n < (interval ? 3 : 0) ||
                (Integer)n > (interval ? 60 : 250)) throw new JSONException(key);
        }
        JSONArray rows=v.has("connections")?v.getJSONArray("connections"):new JSONArray();
        if(rows.length()>24)throw new JSONException("connections");
        for(int i=0;i<rows.length();i++){
            JSONObject row=rows.getJSONObject(i);
            if(row.length()!=2 || !row.getString("kind").matches("[a-z][a-z0-9-]{0,31}")) throw new JSONException("kind");
            row.put("address",address(row.getString("address"),row.getString("kind")));
        }
        return v.put("connections",rows).put("format","garlic-plugin-data/1");
    }
    static String read(InputStream in) throws IOException {
        try (InputStream stream = in; ByteArrayOutputStream bytes = new ByteArrayOutputStream()) {
            if (stream == null) throw new IOException();
            byte[] buf = new byte[2048]; int n;
            while ((n = stream.read(buf)) != -1) {
                if (bytes.size()+n > 16384) throw new IOException();
                bytes.write(buf,0,n);
            }
            return bytes.toString("UTF-8");
        }
    }
    static JSONObject load(Activity a) throws Exception {
        AtomicFile file = new AtomicFile(state(a));
        return parse(read(file.openRead()));
    }
    static void save(Activity a, JSONObject data) throws Exception {
        AtomicFile file = new AtomicFile(state(a));
        FileOutputStream out = file.startWrite();
        try { out.write(data.toString().getBytes(StandardCharsets.UTF_8)); file.finishWrite(out); }
        catch (Exception e) { file.failWrite(out); throw e; }
    }
    static final class Slot {
        final File state;final String name;final boolean enabled;
        Slot(File state,String name,boolean enabled){this.state=state;this.name=name;this.enabled=enabled;}
        void enable(boolean on)throws Exception{
            AtomicFile f=new AtomicFile(state);
            JSONObject v=parse(read(f.openRead()));v.put("enabled",on);
            FileOutputStream out=f.startWrite();
            try{out.write(v.toString().getBytes(StandardCharsets.UTF_8));f.finishWrite(out);}
            catch(Exception e){f.failWrite(out);throw e;}
        }
    }
    static java.util.List<Slot> networkSlots(Activity a){
        java.util.ArrayList<Slot> result=new java.util.ArrayList<>();
        File[] dirs=new File(folder(a,null),"plugins-v3").listFiles();
        if(dirs!=null){java.util.Arrays.sort(dirs);for(File dir:dirs)try{
            JSONObject m=new JSONObject(read(new AtomicFile(new File(dir,"client.gpf-module")).openRead()));
            if(!"network".equals(m.optString("capability")) || !dir.getName().equals("s-"+m.getString("id")))continue;
            File path=new File(dir,"client.installed-gpf");
            JSONObject v=parse(read(new AtomicFile(path).openRead()));
            if(v.getString("id").equals(m.getString("id")))result.add(new Slot(path,m.getString("name"),v.getBoolean("enabled")));
        }catch(Exception ignored){}}
        return result;
    }
    static void show(Activity a) {
        a.startActivity(new Intent(a,PluginPage.class));
    }
    static void importSettings(Activity a, String input) throws Exception {
        JSONObject document = new JSONObject(input);
        Object supplied = document.remove("credentials");
        JSONObject settings = parse(document.toString());
        if (!manifest(a).getString("id").equals(settings.getString("id"))) throw new IOException();
        if (supplied != null) {
            if (!(supplied instanceof JSONObject)) throw new IOException();
            JSONObject creds = (JSONObject)supplied;
            AtomicFile store = new AtomicFile(new File(folder(a), "client.connector-login"));
            JSONObject saved = store.getBaseFile().exists() ? new JSONObject(read(store.openRead())) : new JSONObject();
            JSONArray rows = settings.getJSONArray("connections");
            java.util.Set<String> allowed = new java.util.HashSet<>();
            for (int i=0; i<rows.length(); i++) {
                JSONObject row=rows.getJSONObject(i);
                if (!row.getString("kind").matches("direct|ingress")) allowed.add(row.getString("address"));
            }
            for (java.util.Iterator<String> it=creds.keys(); it.hasNext();) {
                String endpoint=it.next();
                if (!allowed.contains(endpoint)) throw new IOException();
                JSONObject login=creds.getJSONObject(endpoint);
                if (login.length()!=3) throw new IOException();
                for (String field:new String[]{"user","password","pin"})
                    if (!(login.get(field) instanceof String)) throw new IOException();
                saved.put(endpoint,login);
            }
            FileOutputStream out=store.startWrite();
            try {out.write(saved.toString().getBytes(StandardCharsets.UTF_8));store.finishWrite(out);}
            catch(Exception e){store.failWrite(out);throw e;}
        }
        save(a,settings);
    }
    static void result(Activity a,int request,int result,Intent data) {
        if(request<INSTALL || request>EXPORT || result!=Activity.RESULT_OK || data==null || data.getData()==null) return;
        new Thread(() -> {
            try {
                if(request!=INSTALL && !state(a).isFile()) throw new IOException();
                
                if(request==EXPORT) {
                    byte[] bytes=load(a).toString(2).getBytes(StandardCharsets.UTF_8);
                    try(OutputStream out=a.getContentResolver().openOutputStream(data.getData(),"wt")) {
                        if(out==null) throw new IOException();
                        out.write(bytes);
                    }
                } else {
                    String input=read(a.getContentResolver().openInputStream(data.getData()));
                    if(request==INSTALL){a.runOnUiThread(()->{try{importPackage((PluginPage)a,input);}catch(Exception e){toast(a,"Неверный пакет или подпись");}});return;}
                    else {importSettings(a,input);}
                }
                a.runOnUiThread(() -> { if(a instanceof PluginPage)((PluginPage)a).render(); toast(a,request==EXPORT?"Данные сохранены":"Готово"); });
            } catch(Exception e) { a.runOnUiThread(() -> toast(a,"Ошибка файла плагина")); }
        },"garlic-plugin-file").start();
    }
    static void toast(Activity a,String text) { Toast.makeText(a,text,Toast.LENGTH_LONG).show(); }
}
