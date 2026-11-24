.class public final synthetic Lorg/garlic/client/MainActivity$30;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/view/View$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$showServers$28"
    proto = "(Ljava/lang/String;Lorg/json/JSONObject;IILandroid/view/View;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Ljava/lang/String;

.field public final synthetic f$2:Lorg/json/JSONObject;

.field public final synthetic f$3:I

.field public final synthetic f$4:I


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;Lorg/json/JSONObject;II)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$30;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$30;->f$1:Ljava/lang/String;

    iput-object p3, p0, Lorg/garlic/client/MainActivity$30;->f$2:Lorg/json/JSONObject;

    iput p4, p0, Lorg/garlic/client/MainActivity$30;->f$3:I

    iput p5, p0, Lorg/garlic/client/MainActivity$30;->f$4:I

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/view/View;)V
    .locals 6

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$30;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$30;->f$1:Ljava/lang/String;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$30;->f$2:Lorg/json/JSONObject;

    iget v3, p0, Lorg/garlic/client/MainActivity$30;->f$3:I

    iget v4, p0, Lorg/garlic/client/MainActivity$30;->f$4:I

    move-object v5, p1

    invoke-virtual/range {v0 .. v5}, Lorg/garlic/client/MainActivity;->lambda$showServers$28$org-garlic-client-MainActivity(Ljava/lang/String;Lorg/json/JSONObject;IILandroid/view/View;)V

    return-void
.end method
