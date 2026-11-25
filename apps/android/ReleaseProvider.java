package org.garlic.client;

import android.content.ContentProvider;
import android.content.ContentValues;
import android.database.Cursor;
import android.database.MatrixCursor;
import android.net.Uri;
import android.os.ParcelFileDescriptor;
import android.provider.OpenableColumns;
import java.io.FileNotFoundException;

public final class ReleaseProvider extends ContentProvider {
    private void check(Uri uri) {
        if (!"/update.apk".equals(uri.getPath()) || !(getContext().getPackageName()+".release").equals(uri.getAuthority())) throw new IllegalArgumentException();
    }
    public boolean onCreate(){ return true; }
    public String getType(Uri uri){ check(uri);return "application/vnd.android.package-archive"; }
    public ParcelFileDescriptor openFile(Uri uri,String mode) throws FileNotFoundException {
        check(uri); if(!"r".equals(mode)) throw new FileNotFoundException();
        return ParcelFileDescriptor.open(ReleaseUpdate.pending(getContext()),ParcelFileDescriptor.MODE_READ_ONLY);
    }
    public Cursor query(Uri uri,String[] fields,String where,String[] args,String order) {
        check(uri);if(fields==null) fields=new String[]{OpenableColumns.DISPLAY_NAME,OpenableColumns.SIZE};
        MatrixCursor cursor=new MatrixCursor(fields);Object[] row=new Object[fields.length];
        for(int i=0;i<fields.length;i++) {
            if(OpenableColumns.DISPLAY_NAME.equals(fields[i])) row[i]="garlic-alpha.apk";
            if(OpenableColumns.SIZE.equals(fields[i])) row[i]=ReleaseUpdate.pending(getContext()).length();
        }
        cursor.addRow(row);return cursor;
    }
    public Uri insert(Uri uri,ContentValues v){throw new UnsupportedOperationException();}
    public int delete(Uri uri,String w,String[] a){throw new UnsupportedOperationException();}
    public int update(Uri uri,ContentValues v,String w,String[] a){throw new UnsupportedOperationException();}
}
