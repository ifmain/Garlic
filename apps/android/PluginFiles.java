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
    static File state(Activity a) { return new File(a.getFilesDir(), "garlic/client.installed-gpf"); }
    static File module(Activity a) { return new File(a.getFilesDir(),"garlic/client.gpf-module"); }
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
    static String address(String value) throws JSONException {
        try {
            java.net.URI u=new java.net.URI(value);
            if (!"https".equals(u.getScheme()) || u.getHost()==null || u.getUserInfo()!=null || u.getQuery()!=null ||
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
            if(row.length()!=2 || !("direct".equals(row.get("kind")) || "ingress".equals(row.get("kind")))) throw new JSONException("kind");
            row.put("address",address(row.getString("address")));
        }
        return v.put("connections",rows).put("format","garlic-plugin-data/1");
    }
    static String read(InputStream in) throws IOException {
        try (InputStream stream = in; ByteArrayOutputStream bytes = new ByteArrayOutputStream()) {
            if (stream == null) throw new IOException();
            byte[] buf = new byte[2048]; int n;
            while ((n = stream.read(buf)) != -1) {
                if (bytes.size()+n > 8192) throw new IOException();
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
    static void show(Activity a) {
        a.startActivity(new Intent(a,PluginPage.class));
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
                    if(request==INSTALL)install(a,input);
                    else {JSONObject imported=parse(input);if(!manifest(a).getString("id").equals(imported.getString("id")))throw new IOException();save(a,imported);}
                }
                a.runOnUiThread(() -> { if(a instanceof PluginPage)((PluginPage)a).render(); toast(a,request==EXPORT?"Данные сохранены":"Готово"); });
            } catch(Exception e) { a.runOnUiThread(() -> toast(a,"Ошибка файла плагина")); }
        },"garlic-plugin-file").start();
    }
    static void toast(Activity a,String text) { Toast.makeText(a,text,Toast.LENGTH_LONG).show(); }
}
