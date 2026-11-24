.class public final Lorg/garlic/client/GarlicApplication;
.super Landroid/app/Application;
.source "GarlicApplication.java"


# direct methods
.method public constructor <init>()V
    .locals 0

    .line 5
    invoke-direct {p0}, Landroid/app/Application;-><init>()V

    return-void
.end method


# virtual methods
.method public onCreate()V
    .locals 4

    .line 7
    invoke-super {p0}, Landroid/app/Application;->onCreate()V

    .line 9
    :try_start_0
    const-string v0, "GARLIC_APP_VERSION"

    invoke-virtual {p0}, Lorg/garlic/client/GarlicApplication;->getPackageManager()Landroid/content/pm/PackageManager;

    move-result-object v1

    invoke-virtual {p0}, Lorg/garlic/client/GarlicApplication;->getPackageName()Ljava/lang/String;

    move-result-object v2

    const/4 v3, 0x0

    invoke-virtual {v1, v2, v3}, Landroid/content/pm/PackageManager;->getPackageInfo(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;

    move-result-object v1

    iget-object v1, v1, Landroid/content/pm/PackageInfo;->versionName:Ljava/lang/String;

    const/4 v2, 0x1

    invoke-static {v0, v1, v2}, Landroid/system/Os;->setenv(Ljava/lang/String;Ljava/lang/String;Z)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    .line 10
    :catch_0
    move-exception v0

    :goto_0
    nop

    .line 11
    invoke-static {p0}, Lorg/garlic/client/CoreLoader;->initialize(Landroid/content/Context;)V

    .line 12
    invoke-static {p0}, Lorg/garlic/client/InterfaceFactory;->initialize(Landroid/app/Application;)V

    .line 13
    return-void
.end method
