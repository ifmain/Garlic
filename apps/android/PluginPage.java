package org.garlic.client;

import android.app.*;
import android.content.*;
import android.graphics.Color;
import android.graphics.drawable.GradientDrawable;
import android.os.Bundle;
import android.view.*;
import android.widget.*;
import org.json.*;

public final class PluginPage extends Activity {
    private LinearLayout page;
    private boolean details;
    private JSONObject config;
    int dp(int n) { return Math.round(getResources().getDisplayMetrics().density*n); }
    @Override public void onCreate(Bundle state) { super.onCreate(state); details=state!=null && state.getBoolean("details"); render(); }
    @Override public void onSaveInstanceState(Bundle b) { super.onSaveInstanceState(b); b.putBoolean("details",details); }
    @Override public void onBackPressed() { if(details){details=false; render();} else super.onBackPressed(); }
    @Override protected void onActivityResult(int request,int result,Intent data) {
        super.onActivityResult(request,result,data); PluginFiles.result(this,request,result,data);
    }
    void label(String s,int size) { TextView t=new TextView(this); t.setText(s); t.setTextSize(size); t.setTextColor(0xffeeeeee); t.setPadding(0,dp(12),0,dp(8)); page.addView(t); }
    interface Action { void run() throws Exception; }
    Button button(LinearLayout parent,String title,Action action) {
        Button b=new Button(this); b.setText(title); b.setAllCaps(false); b.setTextColor(Color.WHITE);
        GradientDrawable bg=new GradientDrawable(); bg.setColor(0xff292929); bg.setCornerRadius(dp(12)); b.setBackground(bg);
        LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(parent==page?-1:-2,dp(48)); lp.setMargins(0,dp(4),dp(6),dp(4));parent.addView(b,lp);
        b.setOnClickListener(v->{try{action.run();}catch(Exception e){PluginFiles.toast(this,"Проверьте данные подключения");}});return b;
    }
    void write() throws Exception { PluginFiles.save(this,config); render(); }
    void pick(int code) {
        boolean export=code==PluginFiles.EXPORT;
        Intent i=new Intent(export?Intent.ACTION_CREATE_DOCUMENT:Intent.ACTION_OPEN_DOCUMENT);
        i.addCategory(Intent.CATEGORY_OPENABLE).setType(export?"application/json":"*/*");
        if(export)i.putExtra(Intent.EXTRA_TITLE,"plugin-data.json"); startActivityForResult(i,code);
    }
    void render() {
        ScrollView scroll=new ScrollView(this);scroll.setBackgroundColor(0xff181818);page=new LinearLayout(this);page.setOrientation(1);
        page.setPadding(dp(20),dp(24),dp(20),dp(24));scroll.addView(page);setContentView(scroll);
        boolean installed=PluginFiles.module(this).isFile(); if(!installed)details=false;
        button(page,details?"‹  Плагины":"‹  Garlic",()->onBackPressed());
        label(details?PluginFiles.title(this):"Плагины",28);
        if(!installed){label("Нет установленных плагинов",16);button(page,"Установить GPF",()->pick(PluginFiles.INSTALL));return;}
        try {
            config=PluginFiles.load(this);
            if(!details){
                label(PluginFiles.title(this),21);LinearLayout row=new LinearLayout(this);page.addView(row);
                Switch toggle=new Switch(this);toggle.setText("Включён   ");toggle.setChecked(config.getBoolean("enabled"));row.addView(toggle);
                toggle.setOnCheckedChangeListener((v,on)->{try{config.put("enabled",on); write();}catch(Exception e){render();}});
                button(row,"⚙  Настройки",()->{details=true;render();});return;
            }
            label("Подключения",20);label("Используются при следующем подключении.",14);
            JSONArray rows=config.getJSONArray("connections");
            for(int i=0;i<rows.length();i++){final int index=i;JSONObject row=rows.getJSONObject(i);label(row.getString("address"),16);
                button(page,(row.getString("kind").equals("direct")?"Напрямую":"Через входную ноду")+"   ·   Изменить",()->editor(index));}
            button(page,"＋  Добавить подключение",()->editor(-1));
            label("Данные плагина",20);button(page,"Импорт JSON",()->pick(PluginFiles.IMPORT));button(page,"Экспорт JSON",()->pick(PluginFiles.EXPORT));
            button(page,"Удалить плагин",()->new AlertDialog.Builder(this).setMessage("Удалить плагин и его настройки?")
                .setNegativeButton("Отмена",null).setPositiveButton("Удалить",(d,w)->{new java.io.File(getFilesDir(),"garlic/plugin/client.sessions.json").delete();new android.util.AtomicFile(PluginFiles.module(this)).delete();new android.util.AtomicFile(PluginFiles.state(this)).delete();render();}).show());
        }catch(Exception e){label("Не удалось прочитать настройки",16);button(page,"Восстановить из JSON",()->pick(PluginFiles.IMPORT));}
    }
    void editor(int index) throws Exception {
        JSONArray rows=config.getJSONArray("connections"); JSONObject old=index<0?null:rows.getJSONObject(index);
        LinearLayout box=new LinearLayout(this);box.setOrientation(1);box.setPadding(dp(20),dp(8),dp(20),0);
        Spinner kind=new Spinner(this);ArrayAdapter<String> types=new ArrayAdapter<>(this,android.R.layout.simple_spinner_dropdown_item,new String[]{"Garlic · напрямую","Garlic · входная нода"});kind.setAdapter(types);box.addView(kind);
        EditText addr=new EditText(this);addr.setSingleLine();addr.setInputType(17);addr.setHint("https://server.example:8443");box.addView(addr);
        EditText key=new EditText(this);key.setSingleLine();key.setInputType(129);key.setSaveEnabled(false);key.setHint("Ключ Garlic · пусто для сохранённого");box.addView(key);
        if(old!=null){addr.setText(old.getString("address"));kind.setSelection(old.getString("kind").equals("direct")?0:1);}
        AlertDialog.Builder dlg=new AlertDialog.Builder(this).setTitle(index<0?"Добавить подключение":"Подключение").setView(box).setNegativeButton("Отмена",null).setPositiveButton("Сохранить",null);
        if(index>=0)dlg.setNeutralButton("Удалить",(d,w)->{rows.remove(index);try{write();}catch(Exception e){render();}});
        AlertDialog d=dlg.show();d.getButton(-1).setOnClickListener(v->{try{
            String address=PluginFiles.address(addr.getText().toString().trim());
            if(index<0 && rows.length()>=24)throw new Exception();
            if(key.length()>0) {
                java.io.File dir=new java.io.File(getFilesDir(),"garlic/plugin");
                Object reply=Class.forName("org.garlic.client.GarlicBridge").getMethod("saveKey",String.class,String.class,String.class)
                    .invoke(null,dir.getAbsolutePath(),address,key.getText().toString());
                if(!new JSONObject((String)reply).optBoolean("ok"))throw new Exception();
                key.setText("");
            }
            rows.put(index<0?rows.length():index,new JSONObject().put("kind",kind.getSelectedItemPosition()==0?"direct":"ingress").put("address",address));
            write();d.dismiss();
        }catch(Exception e){addr.setError("Введите HTTPS-адрес сервера без ключа, пути и параметров");}});
    }
}
