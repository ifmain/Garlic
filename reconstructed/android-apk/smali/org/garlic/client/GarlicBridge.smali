.class public final Lorg/garlic/client/GarlicBridge;
.super Ljava/lang/Object;
.source "GarlicBridge.java"


# static fields
.field public static volatile loadError:Ljava/lang/String;


# direct methods
.method static constructor <clinit>()V
    .locals 1

    .line 7
    const/4 v0, 0x0

    sput-object v0, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    return-void
.end method

.method private constructor <init>()V
    .locals 0

    .line 9
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method

.method public static native account(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native adminListUsers(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native adminManageUser(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native adminNetwork(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native connectNetwork(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native generatePac(Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native loginMulti(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native saveKey(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native startProxy(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method

.method public static native startTun(I)Ljava/lang/String;
.end method

.method public static native status()Ljava/lang/String;
.end method

.method public static native stopProxy()Ljava/lang/String;
.end method

.method public static native takeover(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
.end method
