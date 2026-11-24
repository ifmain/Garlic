.class public final synthetic Lorg/garlic/client/MainActivity$12;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$userActions$48"
    proto = "(Lorg/json/JSONObject;Ljava/lang/String;ZLandroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Lorg/json/JSONObject;

.field public final synthetic f$2:Ljava/lang/String;

.field public final synthetic f$3:Z


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;Ljava/lang/String;Z)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$12;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$12;->f$1:Lorg/json/JSONObject;

    iput-object p3, p0, Lorg/garlic/client/MainActivity$12;->f$2:Ljava/lang/String;

    iput-boolean p4, p0, Lorg/garlic/client/MainActivity$12;->f$3:Z

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 6

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$12;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$12;->f$1:Lorg/json/JSONObject;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$12;->f$2:Ljava/lang/String;

    iget-boolean v3, p0, Lorg/garlic/client/MainActivity$12;->f$3:Z

    move-object v4, p1

    move v5, p2

    invoke-virtual/range {v0 .. v5}, Lorg/garlic/client/MainActivity;->lambda$userActions$48$org-garlic-client-MainActivity(Lorg/json/JSONObject;Ljava/lang/String;ZLandroid/content/DialogInterface;I)V

    return-void
.end method
