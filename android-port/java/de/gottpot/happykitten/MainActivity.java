package de.gottpot.happykitten;

import android.app.NativeActivity;
import android.content.ClipData;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.provider.MediaStore;
import android.provider.OpenableColumns;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

/**
 * Die Rust-App läuft als NativeActivity. Diese Klasse ergänzt, was nur Java kann:
 * die System-Dateiauswahl (Dateien werden in den App-Ordner kopiert, Rust bekommt ganz normale
 * Pfade) und das Ablegen fertiger Videos und Standbilder in der Galerie.
 */
public class MainActivity extends NativeActivity {
    static {
        System.loadLibrary("happy_kitten");
    }

    private static final int REQ_PICK = 4711;
    private int pickPurpose = -1;

    // Antworten an Rust (siehe app/src/files.rs)
    static native void nativePicked(int purpose, String[] paths);

    static native void nativeSaved(boolean ok, String message);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
    }

    /** Ränder von Statusleiste, Kamera-Aussparung und Gesten-/Navigationsleiste in Pixeln: oben, unten, links, rechts. */
    public int[] getInsets() {
        android.view.WindowInsets wi = getWindow().getDecorView().getRootWindowInsets();
        if (wi == null) {
            return new int[4];
        }
        android.graphics.Insets in = wi.getInsets(
                android.view.WindowInsets.Type.systemBars()
                        | android.view.WindowInsets.Type.displayCutout());
        return new int[] {in.top, in.bottom, in.left, in.right};
    }

    /** Kurzes Rütteln: Rückmeldung, dass ein langer Druck gegriffen hat. */
    public void haptic() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                getWindow().getDecorView().performHapticFeedback(
                        android.view.HapticFeedbackConstants.LONG_PRESS);
            }
        });
    }

    /** Öffnet die Dateiauswahl; die Antwort kommt über nativePicked. */
    public void pick(final int purpose, final String[] mimes, final boolean multiple) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
                if (mimes.length == 1) {
                    intent.setType(mimes[0]);
                } else {
                    intent.setType("*/*");
                    intent.putExtra(Intent.EXTRA_MIME_TYPES, mimes);
                }
                pickPurpose = purpose;
                try {
                    startActivityForResult(intent, REQ_PICK);
                } catch (Exception e) {
                    nativePicked(purpose, new String[0]);
                }
            }
        });
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != REQ_PICK) {
            return;
        }
        final int purpose = pickPurpose;
        final List<Uri> uris = new ArrayList<Uri>();
        if (resultCode == RESULT_OK && data != null) {
            ClipData clip = data.getClipData();
            if (clip != null) {
                for (int i = 0; i < clip.getItemCount(); i++) {
                    uris.add(clip.getItemAt(i).getUri());
                }
            } else if (data.getData() != null) {
                uris.add(data.getData());
            }
        }
        // Kopieren kann bei großen Videos dauern: nicht auf dem UI-Thread.
        new Thread(new Runnable() {
            @Override
            public void run() {
                List<String> paths = new ArrayList<String>();
                long stamp = System.currentTimeMillis();
                for (int i = 0; i < uris.size(); i++) {
                    File dir = new File(getFilesDir(), "medien/" + stamp + "-" + i);
                    String path = copyToApp(uris.get(i), dir);
                    if (path != null) {
                        paths.add(path);
                    }
                }
                nativePicked(purpose, paths.toArray(new String[0]));
            }
        }, "hk-copy").start();
    }

    private String displayName(Uri uri) {
        String name = null;
        Cursor c = getContentResolver().query(uri, null, null, null, null);
        if (c != null) {
            try {
                int col = c.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                if (c.moveToFirst() && col >= 0) {
                    name = c.getString(col);
                }
            } finally {
                c.close();
            }
        }
        if (name == null || name.length() == 0) {
            name = "datei";
        }
        return name.replace('/', '_');
    }

    private String copyToApp(Uri uri, File dir) {
        if (!dir.mkdirs() && !dir.isDirectory()) {
            return null;
        }
        File out = new File(dir, displayName(uri));
        try (InputStream in = getContentResolver().openInputStream(uri);
                OutputStream os = new FileOutputStream(out)) {
            if (in == null) {
                return null;
            }
            byte[] buf = new byte[1 << 16];
            int n;
            while ((n = in.read(buf)) > 0) {
                os.write(buf, 0, n);
            }
            return out.getAbsolutePath();
        } catch (Exception e) {
            return null;
        }
    }

    /** Legt eine Datei in Movies/Happy-Kitten oder Pictures/Happy-Kitten ab. */
    public void saveToGallery(final String path, final String name, final boolean video) {
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    ContentResolver cr = getContentResolver();
                    ContentValues v = new ContentValues();
                    v.put(MediaStore.MediaColumns.DISPLAY_NAME, name);
                    v.put(MediaStore.MediaColumns.MIME_TYPE, video ? "video/mp4" : "image/png");
                    v.put(MediaStore.MediaColumns.RELATIVE_PATH,
                            (video ? Environment.DIRECTORY_MOVIES : Environment.DIRECTORY_PICTURES)
                                    + "/Happy-Kitten");
                    v.put(MediaStore.MediaColumns.IS_PENDING, 1);
                    Uri base = video ? MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                            : MediaStore.Images.Media.EXTERNAL_CONTENT_URI;
                    Uri target = cr.insert(base, v);
                    if (target == null) {
                        nativeSaved(false, "Die Galerie hat die Datei nicht angenommen");
                        return;
                    }
                    try (InputStream in = new FileInputStream(path);
                            OutputStream os = cr.openOutputStream(target)) {
                        byte[] buf = new byte[1 << 16];
                        int n;
                        while ((n = in.read(buf)) > 0) {
                            os.write(buf, 0, n);
                        }
                    }
                    ContentValues done = new ContentValues();
                    done.put(MediaStore.MediaColumns.IS_PENDING, 0);
                    cr.update(target, done, null, null);
                    nativeSaved(true, (video ? "Movies/" : "Pictures/") + "Happy-Kitten/" + name);
                } catch (Exception e) {
                    nativeSaved(false, "Galerie: " + e.getMessage());
                }
            }
        }, "hk-gallery").start();
    }
}
