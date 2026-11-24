.class public final synthetic Lorg/garlic/client/MainActivity$51;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$manage$52"
    proto = "(Lorg/json/JSONObject;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Lorg/json/JSONObject;

.field public final synthetic f$2:I


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;I)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$51;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$51;->f$1:Lorg/json/JSONObject;

    iput p3, p0, Lorg/garlic/client/MainActivity$51;->f$2:I

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$51;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$51;->f$1:Lorg/json/JSONObject;

    iget v2, p0, Lorg/garlic/client/MainActivity$51;->f$2:I

    invoke-virtual {v0, v1, v2}, Lorg/garlic/client/MainActivity;->lambda$manage$52$org-garlic-client-MainActivity(Lorg/json/JSONObject;I)V

    return-void
.end method
