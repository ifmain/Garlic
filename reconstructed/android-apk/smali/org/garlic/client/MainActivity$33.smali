.class public final synthetic Lorg/garlic/client/MainActivity$33;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$subscriptionDialog$45"
    proto = "(Ljava/util/Calendar;Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/CheckBox;Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Ljava/util/Calendar;

.field public final synthetic f$2:Ljava/lang/String;

.field public final synthetic f$3:Landroid/widget/EditText;

.field public final synthetic f$4:Landroid/widget/CheckBox;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Ljava/util/Calendar;Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/CheckBox;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$33;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$33;->f$1:Ljava/util/Calendar;

    iput-object p3, p0, Lorg/garlic/client/MainActivity$33;->f$2:Ljava/lang/String;

    iput-object p4, p0, Lorg/garlic/client/MainActivity$33;->f$3:Landroid/widget/EditText;

    iput-object p5, p0, Lorg/garlic/client/MainActivity$33;->f$4:Landroid/widget/CheckBox;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 7

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$33;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$33;->f$1:Ljava/util/Calendar;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$33;->f$2:Ljava/lang/String;

    iget-object v3, p0, Lorg/garlic/client/MainActivity$33;->f$3:Landroid/widget/EditText;

    iget-object v4, p0, Lorg/garlic/client/MainActivity$33;->f$4:Landroid/widget/CheckBox;

    move-object v5, p1

    move v6, p2

    invoke-virtual/range {v0 .. v6}, Lorg/garlic/client/MainActivity;->lambda$subscriptionDialog$45$org-garlic-client-MainActivity(Ljava/util/Calendar;Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/CheckBox;Landroid/content/DialogInterface;I)V

    return-void
.end method
