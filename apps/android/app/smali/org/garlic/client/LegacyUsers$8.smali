.class public final synthetic Lorg/garlic/client/LegacyUsers$8;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$create$8"
    proto = "(Landroid/widget/Button;Ljava/util/Calendar;)V"
.end annotation


# instance fields
.field public final synthetic f$0:Landroid/widget/Button;

.field public final synthetic f$1:Ljava/util/Calendar;


# direct methods
.method public synthetic constructor <init>(Landroid/widget/Button;Ljava/util/Calendar;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$8;->f$0:Landroid/widget/Button;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers$8;->f$1:Ljava/util/Calendar;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 2

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$8;->f$0:Landroid/widget/Button;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers$8;->f$1:Ljava/util/Calendar;

    invoke-static {v0, v1}, Lorg/garlic/client/LegacyUsers;->lambda$create$8(Landroid/widget/Button;Ljava/util/Calendar;)V

    return-void
.end method
