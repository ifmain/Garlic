.class public final synthetic Lorg/garlic/client/MainActivity$9;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/util/function/BooleanSupplier;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$showLegacy$25"
    proto = "(I)Z"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:I


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;I)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$9;->f$0:Lorg/garlic/client/MainActivity;

    iput p2, p0, Lorg/garlic/client/MainActivity$9;->f$1:I

    return-void
.end method


# virtual methods
.method public final getAsBoolean()Z
    .locals 2

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$9;->f$0:Lorg/garlic/client/MainActivity;

    iget v1, p0, Lorg/garlic/client/MainActivity$9;->f$1:I

    invoke-virtual {v0, v1}, Lorg/garlic/client/MainActivity;->lambda$showLegacy$25$org-garlic-client-MainActivity(I)Z

    move-result v0

    return v0
.end method
