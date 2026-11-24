.class public final synthetic Lorg/garlic/client/LegacyUsers$3;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$request$3"
    proto = "(Lorg/json/JSONObject;Ljava/lang/String;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/LegacyUsers;

.field public final synthetic f$1:Lorg/json/JSONObject;

.field public final synthetic f$2:Ljava/lang/String;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/LegacyUsers;Lorg/json/JSONObject;Ljava/lang/String;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$3;->f$0:Lorg/garlic/client/LegacyUsers;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers$3;->f$1:Lorg/json/JSONObject;

    iput-object p3, p0, Lorg/garlic/client/LegacyUsers$3;->f$2:Ljava/lang/String;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$3;->f$0:Lorg/garlic/client/LegacyUsers;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers$3;->f$1:Lorg/json/JSONObject;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers$3;->f$2:Ljava/lang/String;

    invoke-virtual {v0, v1, v2}, Lorg/garlic/client/LegacyUsers;->lambda$request$3$org-garlic-client-LegacyUsers(Lorg/json/JSONObject;Ljava/lang/String;)V

    return-void
.end method
