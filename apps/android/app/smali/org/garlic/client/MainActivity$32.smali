.class public final synthetic Lorg/garlic/client/MainActivity$32;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/widget/CompoundButton$OnCheckedChangeListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$subscriptionDialog$44"
    proto = "(Landroid/widget/Button;Landroid/widget/CompoundButton;Z)V"
.end annotation


# instance fields
.field public final synthetic f$0:Landroid/widget/Button;


# direct methods
.method public synthetic constructor <init>(Landroid/widget/Button;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$32;->f$0:Landroid/widget/Button;

    return-void
.end method


# virtual methods
.method public final onCheckedChanged(Landroid/widget/CompoundButton;Z)V
    .locals 1

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$32;->f$0:Landroid/widget/Button;

    invoke-static {v0, p1, p2}, Lorg/garlic/client/MainActivity;->lambda$subscriptionDialog$44(Landroid/widget/Button;Landroid/widget/CompoundButton;Z)V

    return-void
.end method
