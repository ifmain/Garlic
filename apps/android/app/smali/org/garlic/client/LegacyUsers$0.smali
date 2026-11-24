.class public final synthetic Lorg/garlic/client/LegacyUsers$0;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/app/DatePickerDialog$OnDateSetListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$create$9"
    proto = "(Ljava/util/Calendar;Ljava/lang/Runnable;Landroid/widget/DatePicker;III)V"
.end annotation


# instance fields
.field public final synthetic f$0:Ljava/util/Calendar;

.field public final synthetic f$1:Ljava/lang/Runnable;


# direct methods
.method public synthetic constructor <init>(Ljava/util/Calendar;Ljava/lang/Runnable;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$0;->f$0:Ljava/util/Calendar;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers$0;->f$1:Ljava/lang/Runnable;

    return-void
.end method


# virtual methods
.method public final onDateSet(Landroid/widget/DatePicker;III)V
    .locals 6

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$0;->f$0:Ljava/util/Calendar;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers$0;->f$1:Ljava/lang/Runnable;

    move-object v2, p1

    move v3, p2

    move v4, p3

    move v5, p4

    invoke-static/range {v0 .. v5}, Lorg/garlic/client/LegacyUsers;->lambda$create$9(Ljava/util/Calendar;Ljava/lang/Runnable;Landroid/widget/DatePicker;III)V

    return-void
.end method
