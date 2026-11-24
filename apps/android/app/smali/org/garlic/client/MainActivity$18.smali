.class public final synthetic Lorg/garlic/client/MainActivity$18;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$login$3"
    proto = "(ILandroid/widget/Button;Lorg/json/JSONObject;Ljava/lang/String;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:I

.field public final synthetic f$2:Landroid/widget/Button;

.field public final synthetic f$3:Lorg/json/JSONObject;

.field public final synthetic f$4:Ljava/lang/String;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;ILandroid/widget/Button;Lorg/json/JSONObject;Ljava/lang/String;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$18;->f$0:Lorg/garlic/client/MainActivity;

    iput p2, p0, Lorg/garlic/client/MainActivity$18;->f$1:I

    iput-object p3, p0, Lorg/garlic/client/MainActivity$18;->f$2:Landroid/widget/Button;

    iput-object p4, p0, Lorg/garlic/client/MainActivity$18;->f$3:Lorg/json/JSONObject;

    iput-object p5, p0, Lorg/garlic/client/MainActivity$18;->f$4:Ljava/lang/String;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 5

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$18;->f$0:Lorg/garlic/client/MainActivity;

    iget v1, p0, Lorg/garlic/client/MainActivity$18;->f$1:I

    iget-object v2, p0, Lorg/garlic/client/MainActivity$18;->f$2:Landroid/widget/Button;

    iget-object v3, p0, Lorg/garlic/client/MainActivity$18;->f$3:Lorg/json/JSONObject;

    iget-object v4, p0, Lorg/garlic/client/MainActivity$18;->f$4:Ljava/lang/String;

    invoke-virtual {v0, v1, v2, v3, v4}, Lorg/garlic/client/MainActivity;->lambda$login$3$org-garlic-client-MainActivity(ILandroid/widget/Button;Lorg/json/JSONObject;Ljava/lang/String;)V

    return-void
.end method
