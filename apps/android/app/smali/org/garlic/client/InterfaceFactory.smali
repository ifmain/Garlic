.class public final Lorg/garlic/client/InterfaceFactory;
.super Landroid/app/AppComponentFactory;
.source "InterfaceFactory.java"


# static fields
.field private static final UI:Ljava/lang/String; = "org.garlic.client.ui."

.field private static loader:Ljava/lang/ClassLoader;


# direct methods
.method static bridge synthetic -$$Nest$sfgetloader()Ljava/lang/ClassLoader;
    .locals 1

    sget-object v0, Lorg/garlic/client/InterfaceFactory;->loader:Ljava/lang/ClassLoader;

    return-object v0
.end method

.method public constructor <init>()V
    .locals 0

    .line 18
    invoke-direct {p0}, Landroid/app/AppComponentFactory;-><init>()V

    return-void
.end method

.method private static confirmOnResume(Landroid/app/Application;J)V
    .locals 1

    .line 55
    new-instance v0, Lorg/garlic/client/InterfaceFactory$1;

    invoke-direct {v0, p0, p1, p2}, Lorg/garlic/client/InterfaceFactory$1;-><init>(Landroid/app/Application;J)V

    invoke-virtual {p0, v0}, Landroid/app/Application;->registerActivityLifecycleCallbacks(Landroid/app/Application$ActivityLifecycleCallbacks;)V

    .line 70
    return-void
.end method

.method static initialize(Landroid/app/Application;)V
    .locals 13

    .line 23
    const-string v0, "trial"

    const-string v1, "path"

    const/4 v2, 0x0

    move v3, v2

    :goto_0
    const/4 v4, 0x2

    if-ge v3, v4, :cond_8

    .line 24
    nop

    .line 26
    const/4 v4, 0x0

    :try_start_0
    const-string v5, "select"

    invoke-static {p0, v5}, Lorg/garlic/client/CoreLoader;->interfaceUpdate(Landroid/content/Context;Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v4

    .line 27
    invoke-virtual {v4, v1}, Lorg/json/JSONObject;->has(Ljava/lang/String;)Z

    move-result v5

    if-nez v5, :cond_0

    return-void

    .line 28
    :cond_0
    new-instance v5, Ljava/io/File;

    invoke-virtual {v4, v1}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v6

    invoke-direct {v5, v6}, Ljava/io/File;-><init>(Ljava/lang/String;)V

    invoke-virtual {v5}, Ljava/io/File;->toPath()Ljava/nio/file/Path;

    move-result-object v5

    .line 29
    invoke-static {v5}, Ljava/nio/file/Files;->size(Ljava/nio/file/Path;)J

    move-result-wide v6

    .line 30
    const-wide/16 v8, 0x0

    cmp-long v8, v6, v8

    if-lez v8, :cond_5

    const-wide/32 v8, 0x800000

    cmp-long v6, v6, v8

    if-gtz v6, :cond_5

    .line 31
    invoke-static {v5}, Ljava/nio/file/Files;->readAllBytes(Ljava/nio/file/Path;)[B

    move-result-object v5

    .line 32
    new-instance v6, Ljava/lang/StringBuilder;

    invoke-direct {v6}, Ljava/lang/StringBuilder;-><init>()V

    .line 33
    const-string v7, "SHA-256"

    invoke-static {v7}, Ljava/security/MessageDigest;->getInstance(Ljava/lang/String;)Ljava/security/MessageDigest;

    move-result-object v7

    invoke-virtual {v7, v5}, Ljava/security/MessageDigest;->digest([B)[B

    move-result-object v7

    array-length v8, v7

    move v9, v2

    :goto_1
    if-ge v9, v8, :cond_1

    aget-byte v10, v7, v9

    .line 34
    sget-object v11, Ljava/util/Locale;->ROOT:Ljava/util/Locale;

    const-string v12, "%02x"

    and-int/lit16 v10, v10, 0xff

    invoke-static {v10}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;

    move-result-object v10

    filled-new-array {v10}, [Ljava/lang/Object;

    move-result-object v10

    invoke-static {v11, v12, v10}, Ljava/lang/String;->format(Ljava/util/Locale;Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/String;

    move-result-object v10

    invoke-virtual {v6, v10}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    .line 33
    add-int/lit8 v9, v9, 0x1

    goto :goto_1

    .line 36
    :cond_1
    invoke-virtual {v6}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v6

    const-string v7, "sha256"

    invoke-virtual {v4, v7}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v7

    invoke-virtual {v6, v7}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v6

    if-eqz v6, :cond_4

    .line 37
    new-instance v6, Ldalvik/system/InMemoryDexClassLoader;

    invoke-static {v5}, Ljava/nio/ByteBuffer;->wrap([B)Ljava/nio/ByteBuffer;

    move-result-object v5

    invoke-virtual {p0}, Landroid/app/Application;->getClassLoader()Ljava/lang/ClassLoader;

    move-result-object v7

    invoke-direct {v6, v5, v7}, Ldalvik/system/InMemoryDexClassLoader;-><init>(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V

    .line 38
    const-string v5, "org.garlic.client.ui.MainActivity"

    invoke-virtual {v6, v5}, Ljava/lang/ClassLoader;->loadClass(Ljava/lang/String;)Ljava/lang/Class;

    move-result-object v5

    const-class v7, Landroid/app/Activity;

    invoke-virtual {v5, v7}, Ljava/lang/Class;->asSubclass(Ljava/lang/Class;)Ljava/lang/Class;

    move-result-object v5

    .line 39
    const-string v7, "org.garlic.client.ui.GarlicProxyService"

    invoke-virtual {v6, v7}, Ljava/lang/ClassLoader;->loadClass(Ljava/lang/String;)Ljava/lang/Class;

    move-result-object v7

    const-class v8, Landroid/net/VpnService;

    invoke-virtual {v7, v8}, Ljava/lang/Class;->asSubclass(Ljava/lang/Class;)Ljava/lang/Class;

    move-result-object v7

    .line 40
    new-array v8, v2, [Ljava/lang/Class;

    invoke-virtual {v5, v8}, Ljava/lang/Class;->getConstructor([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;

    .line 41
    new-array v8, v2, [Ljava/lang/Class;

    invoke-virtual {v7, v8}, Ljava/lang/Class;->getConstructor([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;

    .line 42
    invoke-virtual {v5}, Ljava/lang/Class;->getClassLoader()Ljava/lang/ClassLoader;

    move-result-object v5

    if-ne v5, v6, :cond_3

    invoke-virtual {v7}, Ljava/lang/Class;->getClassLoader()Ljava/lang/ClassLoader;

    move-result-object v5

    if-ne v5, v6, :cond_3

    .line 44
    invoke-virtual {v4, v0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v5

    if-eqz v5, :cond_2

    const-string v5, "sequence"

    invoke-virtual {v4, v5}, Lorg/json/JSONObject;->getLong(Ljava/lang/String;)J

    move-result-wide v7

    invoke-static {p0, v7, v8}, Lorg/garlic/client/InterfaceFactory;->confirmOnResume(Landroid/app/Application;J)V

    .line 45
    :cond_2
    sput-object v6, Lorg/garlic/client/InterfaceFactory;->loader:Ljava/lang/ClassLoader;

    .line 46
    return-void

    .line 43
    :cond_3
    new-instance v5, Ljava/lang/IllegalStateException;

    const-string v6, "UI entry points"

    invoke-direct {v5, v6}, Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;)V

    throw v5

    .line 36
    :cond_4
    new-instance v5, Ljava/lang/SecurityException;

    const-string v6, "UI digest"

    invoke-direct {v5, v6}, Ljava/lang/SecurityException;-><init>(Ljava/lang/String;)V

    throw v5

    .line 30
    :cond_5
    new-instance v5, Ljava/lang/IllegalArgumentException;

    const-string v6, "UI size"

    invoke-direct {v5, v6}, Ljava/lang/IllegalArgumentException;-><init>(Ljava/lang/String;)V

    throw v5
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0
    .catch Ljava/lang/LinkageError; {:try_start_0 .. :try_end_0} :catch_0

    .line 47
    :catch_0
    move-exception v5

    .line 48
    const-string v6, "Garlic"

    const-string v7, "Interface module unavailable"

    invoke-static {v6, v7, v5}, Landroid/util/Log;->w(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I

    .line 49
    if-eqz v4, :cond_7

    invoke-virtual {v4, v0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v4

    if-nez v4, :cond_6

    goto :goto_2

    .line 23
    :cond_6
    add-int/lit8 v3, v3, 0x1

    goto/16 :goto_0

    .line 49
    :cond_7
    :goto_2
    return-void

    .line 52
    :cond_8
    return-void
.end method


# virtual methods
.method public instantiateActivity(Ljava/lang/ClassLoader;Ljava/lang/String;Landroid/content/Intent;)Landroid/app/Activity;
    .locals 1
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Ljava/lang/InstantiationException;,
            Ljava/lang/IllegalAccessException;,
            Ljava/lang/ClassNotFoundException;
        }
    .end annotation

    .line 74
    sget-object v0, Lorg/garlic/client/InterfaceFactory;->loader:Ljava/lang/ClassLoader;

    if-eqz v0, :cond_0

    const-string v0, "org.garlic.client.MainActivity"

    invoke-virtual {p2, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    .line 75
    sget-object p1, Lorg/garlic/client/InterfaceFactory;->loader:Ljava/lang/ClassLoader;

    const-string p2, "org.garlic.client.ui.MainActivity"

    invoke-super {p0, p1, p2, p3}, Landroid/app/AppComponentFactory;->instantiateActivity(Ljava/lang/ClassLoader;Ljava/lang/String;Landroid/content/Intent;)Landroid/app/Activity;

    move-result-object p1

    return-object p1

    .line 76
    :cond_0
    invoke-super {p0, p1, p2, p3}, Landroid/app/AppComponentFactory;->instantiateActivity(Ljava/lang/ClassLoader;Ljava/lang/String;Landroid/content/Intent;)Landroid/app/Activity;

    move-result-object p1

    return-object p1
.end method

.method public instantiateService(Ljava/lang/ClassLoader;Ljava/lang/String;Landroid/content/Intent;)Landroid/app/Service;
    .locals 1
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Ljava/lang/InstantiationException;,
            Ljava/lang/IllegalAccessException;,
            Ljava/lang/ClassNotFoundException;
        }
    .end annotation

    .line 81
    sget-object v0, Lorg/garlic/client/InterfaceFactory;->loader:Ljava/lang/ClassLoader;

    if-eqz v0, :cond_0

    const-string v0, "org.garlic.client.GarlicProxyService"

    invoke-virtual {p2, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    .line 82
    sget-object p1, Lorg/garlic/client/InterfaceFactory;->loader:Ljava/lang/ClassLoader;

    const-string p2, "org.garlic.client.ui.GarlicProxyService"

    invoke-super {p0, p1, p2, p3}, Landroid/app/AppComponentFactory;->instantiateService(Ljava/lang/ClassLoader;Ljava/lang/String;Landroid/content/Intent;)Landroid/app/Service;

    move-result-object p1

    return-object p1

    .line 83
    :cond_0
    invoke-super {p0, p1, p2, p3}, Landroid/app/AppComponentFactory;->instantiateService(Ljava/lang/ClassLoader;Ljava/lang/String;Landroid/content/Intent;)Landroid/app/Service;

    move-result-object p1

    return-object p1
.end method
